#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    TitleReceived,
    ContentReceived {
        title: String,
    },
    MediaReceived {
        title: String,
        content: String,
    },
}
