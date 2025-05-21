use crate::component::{LocaleSwitch, LightSwitch};
use yew::{function_component, use_state, Callback, Html};

#[derive(Debug, PartialEq, yew::Properties)]
pub struct Props {
    pub scenes: Vec<crate::scene::Scene>,
    pub default_scene_index: usize,
}

#[function_component]
pub fn App(props: &Props) -> Html {
    use {
        crate::component::NotificationManager,
        i18nrs::{yew::I18nProvider, StorageType},
        js_sys::Date,
        std::collections::HashMap,
        yew::{html, virtual_dom::VNode},
    };

    let scenes = props.scenes.clone();

    let current_scene_default = {
        scenes
            .get(props.default_scene_index)
            .or_else(|| scenes.first())
            .cloned()
            .unwrap()
    };

    let current_scene = use_state(|| current_scene_default);

    html! {
        <I18nProvider
            // There is probably a way to put theses in a use_state and have it dynamically load i18ns when
            // swithing to a non-default one
            translations={HashMap::from([
                ("en",include_str!("../resources/i18n/en.json")),
                ("fr",include_str!("../resources/i18n/fr.json"))
            ])}
            storage_type={StorageType::LocalStorage}
            storage_name={"i18nrs".to_string()}
            default_language={"en".to_string()}
        >
        <div id="global">
        <div id="header">
            <a class="header-item" href="http://github.com/Bowarc/wasm_portfolio">
                <img src="/resources/github.webp" alt="Github icon" class="icon"/>
            </a>
            <LocaleSwitch />
            <LightSwitch />
            <div class="header-item" id="scene_list">{
                scenes.into_iter().map(|scene|{
                    html!{
                        <button class={format!("scene_button{}", if *current_scene == scene {" current"} else {""})} onclick={
                            let current_scene_clone = current_scene.clone();
                            Callback::from(move |_| current_scene_clone.set(scene))
                        }>
                            { format!("{scene}") }
                        </button>
                    }
                }).collect::<Vec<VNode>>()
            }</div>
        </div>
        <div id="content">
            {current_scene.html(current_scene.clone())}
            <NotificationManager />
        </div>
        <footer>
            { format!("Rendered: {}", String::from(Date::new_0().to_string())) }
        </footer>
        </div>
        </I18nProvider>
    }
}
