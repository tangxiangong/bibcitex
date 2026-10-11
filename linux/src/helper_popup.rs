//! Separate native candidate surface: never consumes the helper result area's height.
use super::*;

/// Compact candidate geometry shared by initial placement and filtered resizing.
pub(super) fn candidate_size(labels: &[SharedString]) -> Size<Pixels> {
    let width = labels
        .iter()
        .map(|label| {
            label
                .chars()
                .map(|character| if character.is_ascii() { 7usize } else { 14 })
                .sum::<usize>()
                + 28
        })
        .max()
        .unwrap_or(140)
        .clamp(140, 220);
    size(
        px(width as f32),
        px((labels.len().clamp(1, 6) * 28 + 12) as f32),
    )
}

/// Parent-anchored native candidate window, scoped to one picker identity.
pub(super) struct CandidatePopup {
    owner: WeakEntity<Workbench>,
    parent: AnyWindowHandle,
    picker_id: EntityId,
    _subscription: Subscription,
    _activation: Subscription,
}

impl CandidatePopup {
    /// Observe one candidate session; replaced/closed sessions invalidate all callbacks.
    pub(super) fn new(
        owner: Entity<Workbench>,
        parent: AnyWindowHandle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let picker_id = owner
            .read(cx)
            .candidates
            .as_ref()
            .map(|picker| picker.filter.entity_id())
            .unwrap_or(owner.entity_id());
        let subscription = cx.observe(&owner, |_, _, cx| cx.notify());
        let activation = cx.observe_window_activation(window, |_, window, cx| {
            if window.is_window_active() {
                return;
            }
            cx.spawn_in(window, async move |this, cx| {
                cx.background_executor()
                    .timer(Duration::from_millis(30))
                    .await;
                let _ = this.update_in(cx, |this, window, cx| {
                    let composing = this.owner.upgrade().is_some_and(|owner| {
                        owner.update(cx, |owner, cx| owner.input_is_composing(window, cx))
                    });
                    if composing {
                        return;
                    }
                    let parent_active = this
                        .parent
                        .update(cx, |_, parent, _| parent.is_window_active())
                        .unwrap_or(false);
                    if !parent_active && !window.is_window_active() {
                        this.dispatch(
                            |owner, parent, _| {
                                owner.candidates = None;
                                owner.candidate_window = None;
                                parent.remove_window();
                            },
                            cx,
                        );
                        window.remove_window();
                    }
                });
            })
            .detach();
        });
        Self {
            owner: owner.downgrade(),
            parent,
            picker_id,
            _subscription: subscription,
            _activation: activation,
        }
    }

    fn dispatch(
        &self,
        action: impl FnOnce(&mut Workbench, &mut Window, &mut Context<Workbench>) + 'static,
        cx: &mut App,
    ) {
        let owner = self.owner.clone();
        let expected = self.picker_id;
        let _ = self.parent.update(cx, move |_, window, cx| {
            let _ = owner.update(cx, |owner, cx| {
                if owner.input_is_composing(window, cx) {
                    return;
                }
                if owner
                    .candidates
                    .as_ref()
                    .is_some_and(|picker| picker.filter.entity_id() == expected)
                {
                    action(owner, window, cx);
                }
            });
        });
    }
}

impl Render for CandidatePopup {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(owner) = self.owner.upgrade() else {
            window.remove_window();
            return div();
        };
        let view = owner.read(cx);
        let Some(picker) = &view.candidates else {
            window.remove_window();
            return div();
        };
        if picker.filter.entity_id() != self.picker_id {
            window.remove_window();
            return div();
        }
        let input = picker.filter.clone();
        let cursor = picker.cursor;
        let scroll = picker.scroll.clone();
        let rows: Vec<_> = view
            .candidate_items(cx)
            .into_iter()
            .map(|(name, _)| view.candidate_label(picker.trigger, &name, cx))
            .collect();
        let desired = candidate_size(&rows);
        if window.viewport_size() != desired {
            window.resize(desired);
        }
        let empty = view.localized("无候选", "No candidates", cx);
        let mut list = div()
            .id("helper-floating-options")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&scroll)
            .flex()
            .flex_col();
        if rows.is_empty() {
            list = list.child(div().p_2().child(empty));
        }
        for (index, label) in rows.into_iter().enumerate() {
            list = list.child(
                Button::new(("candidate-option", index))
                    .ghost()
                    .small()
                    .h(px(28.))
                    .justify_start()
                    .w_full()
                    .when(index == cursor, |button| button.bg(cx.theme().muted))
                    .label(label)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.dispatch(
                            move |owner, window, cx| {
                                if let Some(picker) = owner.candidates.as_mut() {
                                    picker.cursor = index;
                                }
                                owner.confirm_candidate(window, cx);
                            },
                            cx,
                        );
                    })),
            );
        }
        div()
            .size_full()
            .p(px(6.))
            .flex()
            .flex_col()
            .bg(cx.theme().popover)
            .text_color(cx.theme().foreground)
            .border_1()
            .border_color(cx.theme().border)
            .rounded_lg()
            .capture_key_down(cx.listener(move |this, event: &KeyDownEvent, window, cx| {
                if input.update(cx, |input, cx| {
                    input.marked_text_range(window, cx).is_some()
                }) {
                    return;
                }
                match event.keystroke.key.as_str() {
                    "up" => {
                        this.dispatch(|owner, _, cx| owner.candidate_navigation(-1, cx), cx);
                        cx.stop_propagation();
                    }
                    "down" => {
                        this.dispatch(|owner, _, cx| owner.candidate_navigation(1, cx), cx);
                        cx.stop_propagation();
                    }
                    "escape" => {
                        this.dispatch(
                            |owner, window, cx| {
                                owner.cancel_candidate(window, cx);
                                window.activate_window();
                            },
                            cx,
                        );
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }))
            .child(list)
    }
}
