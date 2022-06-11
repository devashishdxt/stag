use std::rc::Rc;

use yew::{classes, html, Component, Context, Html};

use crate::{
    components::html::Button,
    store::{StoreReader, StoreWriter},
};

use super::NotificationData;

pub enum NotificationMsg {
    NewNotification(Rc<Option<NotificationData>>),
    Clear,
}

pub struct Notification {
    data: Rc<Option<NotificationData>>,
    writer: StoreWriter<Option<NotificationData>>,
    _reader: StoreReader<Option<NotificationData>>,
}

impl Component for Notification {
    type Message = NotificationMsg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            data: Default::default(),
            writer: StoreWriter::new(),
            _reader: StoreReader::new(ctx.link().callback(NotificationMsg::NewNotification)),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_click = ctx.link().callback(|()| NotificationMsg::Clear);

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
        match msg {
            NotificationMsg::NewNotification(data) => {
                self.data = data;
                true
            }
            NotificationMsg::Clear => {
                self.writer.set(None);
                false
            }
        }
    }
}
