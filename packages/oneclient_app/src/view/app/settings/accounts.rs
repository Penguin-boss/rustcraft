use freya::prelude::*;
use oneclient_auth::MinecraftAccount;

use super::{section_header, settings_page};
use crate::components::{Button, Icon, IconType, TextInput};
use crate::hooks::{
    SetOfflineProfileKeys, mutation_error_message, mutation_is_running, try_default_account,
    use_current_account, use_set_offline_profile,
};
use crate::i18n::t;
use crate::theme::{colors, rounded, text_size};
use crate::ui::border_all_color;

const BADGE_SIZE_PX: f32 = 64.;

#[derive(PartialEq)]
pub struct SettingsAccounts;

impl Component for SettingsAccounts {
    fn render(&self) -> impl IntoElement {
        let current = use_current_account();
        let account = try_default_account(&current);
        let save = use_set_offline_profile();

        let initial_name = account
            .as_ref()
            .map(|account| account.username.clone())
            .unwrap_or_default();

        let mut draft = use_state({
            let name = initial_name.clone();
            move || name
        });
        // The query can settle after the first paint, so seed the field once the
        // saved profile arrives instead of leaving it blank.
        let mut seeded = use_state(|| !initial_name.is_empty());
        let seed_name = initial_name.clone();
        use_side_effect(move || {
            if *seeded.read() || seed_name.is_empty() {
                return;
            }
            draft.set(seed_name.clone());
            seeded.set(true);
        });

        let typed = draft.read().trim().to_string();
        let valid = oneclient_auth::validate_offline_username(&typed).is_ok();
        let preview = valid.then(|| oneclient_auth::offline_uuid(&typed).to_string());
        let saving = mutation_is_running(&save);
        let error = mutation_error_message(&save);

        let submit = {
            let save = save.clone();
            move || {
                let name = draft.peek().trim().to_string();
                if name.is_empty() {
                    return;
                }
                save.mutate(SetOfflineProfileKeys { username: name });
            }
        };

        let on_press = submit.clone();
        let on_submit = submit;

        settings_page()
            .child(section_header(t(
                "oneclient.settings.accounts.section.your_accounts",
            )))
            .child(profile_card(account, preview))
            .child(section_header(t(
                "oneclient.settings.accounts.offline.title",
            )))
            .child(
                rect()
                    .vertical()
                    .width(Size::fill())
                    .spacing(12.)
                    .padding(Gaps::new_all(18.))
                    .corner_radius(rounded(12.))
                    .background(colors::page_elevated())
                    .border(border_all_color(1., colors::component_border()))
                    .child(
                        label()
                            .text(t("oneclient.settings.accounts.offline.description"))
                            .font_size(text_size(12.))
                            .color(colors::fg_secondary()),
                    )
                    .child(
                        rect()
                            .horizontal()
                            .width(Size::fill())
                            .cross_align(Alignment::Center)
                            .spacing(10.)
                            .child(
                                TextInput::new(draft)
                                    .placeholder(t(
                                        "oneclient.settings.accounts.offline.username_placeholder",
                                    ))
                                    .width(Size::px(260.))
                                    .on_submit(move |_: String| on_submit()),
                            )
                            .child(
                                Button::new()
                                    .primary()
                                    .disabled(!valid || saving)
                                    .on_press(move |_| on_press())
                                    .text(if saving {
                                        t("oneclient.settings.accounts.offline.saving")
                                    } else {
                                        t("oneclient.settings.accounts.offline.confirm")
                                    }),
                            ),
                    )
                    .child(
                        label()
                            .text(t("oneclient.settings.accounts.offline.hint"))
                            .font_size(text_size(11.))
                            .color(colors::fg_secondary()),
                    )
                    .maybe_child(error.map(|message| {
                        label()
                            .text(message)
                            .font_size(text_size(12.))
                            .color(colors::danger())
                    })),
            )
            .into_element()
    }
}

fn profile_card(account: Option<MinecraftAccount>, preview: Option<String>) -> impl IntoElement {
    let uuid = account.as_ref().map(|account| account.id.to_string());
    let username = account
        .as_ref()
        .map(|account| account.username.clone())
        .unwrap_or_else(|| t("oneclient.settings.accounts.offline.none").to_string());
    let shown_uuid = uuid.or(preview);

    rect()
        .horizontal()
        .width(Size::fill())
        .cross_align(Alignment::Center)
        .spacing(16.)
        .padding(Gaps::new_all(20.))
        .corner_radius(rounded(14.))
        .background(colors::page_elevated())
        .border(border_all_color(1., colors::component_border()))
        .child(
            rect()
                .width(Size::px(BADGE_SIZE_PX))
                .height(Size::px(BADGE_SIZE_PX))
                .center()
                .corner_radius(rounded(14.))
                .background(colors::component_bg())
                .child(
                    Icon::new(IconType::Users01)
                        .size(28.)
                        .color(colors::brand()),
                ),
        )
        .child(
            rect()
                .vertical()
                .spacing(4.)
                .child(
                    label()
                        .text(t("oneclient.settings.accounts.offline.playing_as"))
                        .font_size(text_size(10.))
                        .letter_spacing(0.8)
                        .font_weight(FontWeight::SEMI_BOLD)
                        .color(colors::fg_secondary()),
                )
                .child(
                    label()
                        .text(username)
                        .font_size(text_size(18.))
                        .font_weight(FontWeight::SEMI_BOLD)
                        .color(colors::fg_primary()),
                )
                .maybe_child(shown_uuid.map(|uuid| {
                    label()
                        .text(format!(
                            "{} {uuid}",
                            t("oneclient.settings.accounts.offline.uuid")
                        ))
                        .font_size(text_size(11.))
                        .color(colors::fg_secondary())
                })),
        )
        .into_element()
}
