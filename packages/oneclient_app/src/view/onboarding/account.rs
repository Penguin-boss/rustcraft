use freya::prelude::*;
use oneclient_auth::MinecraftAccount;

use crate::components::{Button, Icon, IconType, TextInput};
use crate::hooks::{
    SetOfflineProfileKeys, mutation_error_message, mutation_is_running, try_default_account,
    use_current_account, use_set_offline_profile,
};
use crate::i18n::t;
use crate::routes::Route;
use crate::theme::{colors, rounded, text_size};
use crate::ui::border_all_color;
use crate::view::onboarding::{
    onboarding_illustration, onboarding_nav, onboarding_page, step_heading,
};

#[derive(PartialEq)]
pub struct OnboardingAccount;

impl Component for OnboardingAccount {
    fn render(&self) -> impl IntoElement {
        let account_query = use_current_account();
        let account = try_default_account(&account_query);
        let save = use_set_offline_profile();

        let initial_name = account
            .as_ref()
            .map(|account| account.username.clone())
            .unwrap_or_default();
        let mut draft = use_state({
            let name = initial_name.clone();
            move || name
        });
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
        let scheduled_uuid = valid.then(|| oneclient_auth::offline_uuid(&typed).to_string());
        let saving = mutation_is_running(&save);
        let error = mutation_error_message(&save);
        let has_account = account.is_some();

        let submit = {
            let save = save.clone();
            move |_: Event<PressEventData>| {
                let name = draft.peek().trim().to_string();
                if name.is_empty() {
                    return;
                }
                save.mutate(SetOfflineProfileKeys { username: name });
            }
        };

        let content = rect()
            .vertical()
            .width(Size::fill())
            .spacing(24.)
            .child(step_heading(
                t("oneclient.onboarding.account.title"),
                t("oneclient.onboarding.account.offline_subtitle"),
            ))
            .child(match &account {
                Some(account) => preview(account).into_element(),
                None => {
                    let submit = submit;
                    form(draft, scheduled_uuid, error, valid, saving, submit).into_element()
                }
            })
            .into_element();

        onboarding_page(
            onboarding_illustration(IconType::OnboardingAccount),
            content,
            onboarding_nav(
                Some(Route::OnboardingLanguage {}),
                Route::OnboardingBundles {},
                has_account,
            ),
        )
    }
}

fn preview(account: &MinecraftAccount) -> impl IntoElement {
    rect()
        .horizontal()
        .width(Size::fill())
        .spacing(14.)
        .cross_align(Alignment::Center)
        .padding(Gaps::new_all(16.))
        .corner_radius(rounded(12.))
        .background(colors::page_elevated())
        .border(border_all_color(1., colors::component_border()))
        .child(
            Icon::new(IconType::CheckCircle)
                .size(22.)
                .color(colors::success()),
        )
        .child(
            rect()
                .vertical()
                .spacing(4.)
                .child(
                    label()
                        .text(account.username.clone())
                        .font_size(text_size(16.))
                        .font_weight(FontWeight::SEMI_BOLD)
                        .color(colors::fg_primary()),
                )
                .child(
                    label()
                        .text(format!(
                            "{} {}",
                            t("oneclient.onboarding.account.uuid"),
                            account.id
                        ))
                        .font_size(text_size(12.))
                        .color(colors::fg_secondary()),
                ),
        )
        .into_element()
}

fn form(
    draft: State<String>,
    scheduled_uuid: Option<String>,
    error: Option<String>,
    valid: bool,
    saving: bool,
    submit: impl FnMut(Event<PressEventData>) + 'static,
) -> impl IntoElement {
    rect()
        .vertical()
        .width(Size::fill())
        .spacing(14.)
        .child(
            label()
                .text(t("oneclient.onboarding.account.offline_description"))
                .font_size(text_size(13.))
                .max_lines(3)
                .color(colors::fg_secondary()),
        )
        .child(
            TextInput::new(draft)
                .placeholder(t("oneclient.onboarding.account.username_placeholder"))
                .width(Size::px(320.)),
        )
        .maybe_child(scheduled_uuid.map(|uuid| {
            label()
                .text(format!("{} {uuid}", t("oneclient.onboarding.account.uuid")))
                .font_size(text_size(11.))
                .color(colors::fg_secondary())
        }))
        .child(
            label()
                .text(t("oneclient.onboarding.account.username_hint"))
                .font_size(text_size(11.))
                .color(colors::fg_secondary()),
        )
        .child(
            Button::new()
                .primary()
                .large()
                .disabled(!valid || saving)
                .on_press(submit)
                .child(Icon::new(IconType::ArrowRight).size(16.))
                .text(if saving {
                    t("oneclient.onboarding.account.saving")
                } else {
                    t("oneclient.onboarding.account.save")
                }),
        )
        .maybe_child(error.map(|message| {
            label()
                .text(message)
                .font_size(text_size(12.))
                .color(colors::danger())
        }))
        .into_element()
}
