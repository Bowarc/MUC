use yew::{function_component, html, Html};

#[derive(yew::Properties, std::cmp::PartialEq)]
pub struct Props {
    pub current_scene: yew::UseStateHandle<crate::scene::Scene>,
}

#[function_component]
pub fn Home(_props: &Props) -> Html {

    if let Some(nav) = yew_router::hooks::use_navigator() {
        nav.replace(&crate::Route::Home)
    }else{
        error!("Failed to retrieve the navigator")
    }

    // let (i18n, _) = i18nrs::yew::use_translation();

    html! { <>
    </>}
}
