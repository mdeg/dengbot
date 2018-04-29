pub struct SlackInfo {
    pub users: Vec<::slack::User>,
    pub listen_channel_id: String,
    pub meta_channel_id: String,
}

// Holds necessary server information received on initial connect
impl SlackInfo {
    pub fn from_start_response(resp: &::slack::api::rtm::StartResponse) -> Self {
        let mut channels = resp.channels
            .as_ref()
            .expect("No channel list returned")
            .iter();

        let listen_channel_id = channels
            .find(|channel| channel.name.as_ref().expect("No listen channel name found") == ::LISTEN_CHANNEL_NAME)
            .expect("Could not find listen channel by that name")
            .id
            .clone()
            .expect("No ID associated with listen channel");

        debug!("Found listen channel ID: {}", listen_channel_id);

        let meta_channel_id = channels
            .find(|channel| {
                channel.name.as_ref().expect("No listen channel name found") == ::POST_CHANNEL_NAME
            })
            .expect("Could not find meta channel by that name")
            .id
            .clone()
            .expect("No ID associated with meta channel");

        debug!("Found meta channel ID: {}", meta_channel_id);

        trace!("Unfiltered users list: {:?}", resp.users.as_ref().unwrap());

        let users = resp.users.clone().expect("No users returned on connection");

        SlackInfo {
            users,
            listen_channel_id,
            meta_channel_id,
        }
    }
}
