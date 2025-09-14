use uuid::Uuid;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    TitleReceive,
    ContentReceive {
        title: String,
    },
    MediaReceive {
        title: String,
        content: String,
    },
    PublishDateReceive {
        post_id: Uuid,
    },
}
