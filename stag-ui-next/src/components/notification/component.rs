use yew::{classes, html, Component, Context, Html};
use yew_agent::{
    utils::store::{ReadOnly, StoreWrapper},
    Bridge, Bridged,
};

use crate::components::html::Button;

use super::{store::NotificationStore, NotificationData};

pub struct Notification {
    data: Option<NotificationData>,
    _bridge: Box<dyn Bridge<StoreWrapper<NotificationStore>>>,
}

impl Component for Notification {
    type Message = Option<NotificationData>;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            data: None,
            _bridge: StoreWrapper::bridge(
                ctx.link()
                    .callback(|store: ReadOnly<NotificationStore>| store.borrow().data.clone()),
            ),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_click = ctx.link().callback(|()| None);

        match self.data.as_ref() {
            None => html! {
                <></>
            },
            Some(data) => html! {
                <div class={classes!("fixed", "left-0", "top-0", "bg-slate-600", "bg-opacity-70", "z-10", "h-screen", "w-screen", "text-center")}>
                    <div class={classes!("inline-block", "px-20", "py-10", "bg-slate-100", "rounded", "mt-60", "text-center")}>
                        <div class={classes!("inline-block", "text-3xl", "pb-4")}>{ data.icon() }</div>
                        <div class={classes!("text-lg", "pb-6")}>{ data.message() }</div>
                        {
                            match data.details() {
                                None => html! {
                                    <></>
                                },
                                Some(details) => html! {
                                    <div class={classes!("text-sm", "pb-6")}>{ details }</div>
                                }
                            }
                        }
                        {
                            if data.is_dismissable() {
                                html! {
                                    <Button text="Dismiss" ty="button" {on_click} />
                                }
                            } else {
                                html! { <></> }
                            }
                        }

                    </div>
                </div>
            },
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        self.data = msg;
        true
    }
}
