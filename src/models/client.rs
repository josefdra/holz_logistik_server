use tokio::sync::mpsc::UnboundedSender;
use warp::ws::Message;

#[derive(Clone)]
pub struct Client {
    pub id: String,
    pub sender: UnboundedSender<Message>,
    pub db_name: String,
    pub user_id: String,
    pub sync_completed: bool,
    pub authenticated: bool,
}
