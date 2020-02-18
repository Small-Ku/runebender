//! The sidebar of the main glyph list/grid view.

use druid::kurbo::Line;
use druid::{
    Affine, BoxConstraints, Color, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx,
    PaintCtx, Rect, RenderContext, Size, UpdateCtx, Widget, WidgetPod,
};

use druid::widget::{Flex, Label, WidgetExt};

use crate::data::{lenses, GlyphPlus, Workspace};
use crate::theme;

const SELECTED_GLYPH_BOTTOM_PADDING: f64 = 10.0;
const SELECTED_GLYPH_HEIGHT: f64 = 220.0;

pub struct Sidebar {
    selected_glyph: WidgetPod<Workspace, Box<dyn Widget<Workspace>>>,
}

impl Sidebar {
    pub fn new() -> Sidebar {
        Sidebar {
            selected_glyph: WidgetPod::new(
                Flex::column()
                    .with_child(
                        Label::new(|d: &Option<GlyphPlus>, _: &Env| {
                            d.as_ref()
                                .map(|d| d.glyph.name.to_string())
                                .unwrap_or("_____".into())
                        })
                        .center(),
                        0.0,
                    )
                    .with_child(
                        Label::new(|d: &Option<GlyphPlus>, _: &Env| {
                            d.as_ref()
                                .and_then(|d| d.codepoint())
                                .map(|c| format!("(U+{:04X})", c as u32))
                                .unwrap_or("______".into())
                        })
                        .center()
                        .env_scope(|env, _| {
                            env.set(druid::theme::LABEL_COLOR, Color::grey8(140));
                            env.set(druid::theme::TEXT_SIZE_NORMAL, 12.0);
                        }),
                        0.0,
                    )
                    .with_child(SelectedGlyph::new().fix_height(100.0).center(), 0.0)
                    .with_child(
                        Label::new(|d: &Option<GlyphPlus>, _: &Env| {
                            d.as_ref()
                                .and_then(|g| {
                                    g.glyph
                                        .advance
                                        .as_ref()
                                        .map(|a| format!("{}", a.width as usize))
                                })
                                .unwrap_or("___".into())
                        })
                        .center(),
                        0.0,
                    )
                    .with_child(
                        Flex::row()
                            .with_child(Label::new("kern group"), 1.0)
                            .with_child(Label::new("kern group"), 1.0),
                        0.0,
                    )
                    .lens(lenses::app_state::SelectedGlyph)
                    .fix_height(SELECTED_GLYPH_HEIGHT)
                    .background(Color::grey8(0xCC))
                    .boxed(),
            ),
        }
    }
}

impl Widget<Workspace> for Sidebar {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Workspace, env: &Env) {
        self.selected_glyph.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &Workspace,
        env: &Env,
    ) {
        self.selected_glyph.lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &Workspace, data: &Workspace, env: &Env) {
        self.selected_glyph.update(ctx, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &Workspace,
        env: &Env,
    ) -> Size {
        let child_size = self.selected_glyph.layout(ctx, bc, data, env);
        let my_size = bc.max();
        let extra_y = my_size.height - child_size.height;
        let extra_x = my_size.width - child_size.width;
        let child_y = (extra_y - SELECTED_GLYPH_BOTTOM_PADDING).max(0.0);
        let child_origin = (extra_x / 2.0, child_y);
        self.selected_glyph
            .set_layout_rect(Rect::from_origin_size(child_origin, child_size));
        my_size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Workspace, env: &Env) {
        let rect = Rect::ZERO.with_size(ctx.size());
        ctx.fill(rect, &env.get(theme::SIDEBAR_BACKGROUND));

        self.selected_glyph.paint_with_offset(ctx, data, env);

        // to get clean strokes we have to *not* align on pixel boundaries
        let max_x = rect.max_x() - 0.5;
        let line = Line::new((max_x, 0.0), (max_x, rect.max_y()));
        ctx.stroke(line, &env.get(theme::SIDEBAR_EDGE_STROKE), 1.0);
    }
}

// currently just paints the glyph shape
struct SelectedGlyph {}

impl SelectedGlyph {
    pub fn new() -> Self {
        SelectedGlyph {}
    }
}

impl Widget<Option<GlyphPlus>> for SelectedGlyph {
    fn event(
        &mut self,
        _ctx: &mut EventCtx,
        _event: &Event,
        _data: &mut Option<GlyphPlus>,
        _env: &Env,
    ) {
    }
    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &Option<GlyphPlus>,
        _env: &Env,
    ) {
    }
    fn update(
        &mut self,
        _ctx: &mut UpdateCtx,
        _old_data: &Option<GlyphPlus>,
        _data: &Option<GlyphPlus>,
        _env: &Env,
    ) {
    }
    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &Option<GlyphPlus>,
        _env: &Env,
    ) -> Size {
        let width = bc.max().width;
        Size::new(width, SELECTED_GLYPH_HEIGHT)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Option<GlyphPlus>, env: &Env) {
        if let Some(data) = data {
            let advance = data
                .glyph
                .advance
                .as_ref()
                .map(|a| a.width as f64)
                .unwrap_or(data.upm() * 0.5);

            let path = data.get_bezier();
            let geom = Rect::ZERO.with_size(ctx.size());
            let scale = geom.height() as f64 / data.upm();
            let scaled_width = advance * scale as f64;
            let l_pad = ((geom.width() as f64 - scaled_width) / 2.).round();
            let baseline = geom.height() - (geom.height() * 0.16) as f64;
            let affine = Affine::new([scale as f64, 0.0, 0.0, -scale as f64, l_pad, baseline]);

            let glyph_color = if data.is_placeholder_glyph() {
                env.get(theme::PLACEHOLDER_GLYPH_COLOR)
            } else {
                env.get(theme::GLYPH_COLOR)
            };

            ctx.fill(affine * &*path, &glyph_color);
        }
    }
}
