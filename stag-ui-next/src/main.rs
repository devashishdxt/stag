mod components;

use components::notification::{NotificationData, NotificationSender};
use yew::{html, Component, Context, Html};

use self::components::{html::Button, notification::Notification};

fn main() {
    yew::start_app::<App>();
}

struct App {
    notification_sender: NotificationSender,
}

impl Component for App {
    type Message = NotificationData;

    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self {
            notification_sender: NotificationSender::new(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_click = ctx.link().callback(|()| NotificationData::Error {
            message: "Error",
            details: Some("Error".to_string()),
        });

        html! {
            <div>
                <Notification />
                <Button text="Click for notification" ty="button" {on_click} />
            </div>
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        self.notification_sender.send(msg);
        false
    }
}
