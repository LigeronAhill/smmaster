pub mod smm {
    pub mod users {
        use anyhow::anyhow;

        tonic::include_proto!("proto.users.v1");

        impl TryFrom<CreateUserRequest> for shared::models::User {
            type Error = anyhow::Error;
            fn try_from(value: CreateUserRequest) -> Result<Self, Self::Error> {
                let u = shared::models::User::builder()
                    .telegram_id(value.telegram_id)
                    .first_name(value.first_name)
                    .last_name(value.last_name)
                    .username(value.username)
                    .language_code(value.language_code)
                    .build()?;
                Ok(u)
            }
        }
        impl From<shared::models::User> for User {
            fn from(value: shared::models::User) -> Self {
                let sc: std::time::SystemTime = value.created_at.into();
                let pc = Some(sc.into());
                let su: std::time::SystemTime = value.updated_at.into();
                let pu = Some(su.into());
                let sl: std::time::SystemTime = value.last_activity.into();
                let pl = Some(sl.into());
                User {
                    id: value.id.to_string(),
                    telegram_id: value.telegram_id,
                    first_name: value.first_name,
                    last_name: value.last_name,
                    username: value.username,
                    language_code: value.language_code,
                    role: value.role.into(),
                    created_at: pc,
                    updated_at: pu,
                    last_activity: pl,
                }
            }
        }
        impl TryFrom<User> for shared::models::User {
            type Error = anyhow::Error;
            fn try_from(value: User) -> Result<Self, Self::Error> {
                let mut b = shared::models::User::builder();
                let role = value.role;
                b.try_id(value.id)?
                    .telegram_id(value.telegram_id)
                    .first_name(value.first_name)
                    .last_name(value.last_name)
                    .username(value.username)
                    .try_role(role)
                    .map_err(|e| anyhow!("{e}"))?
                    .language_code(value.language_code);
                if let Some(c) = value.created_at {
                    b.created_at(c.seconds, c.nanos);
                }
                if let Some(u) = value.updated_at {
                    b.updated_at(u.seconds, u.nanos);
                }
                if let Some(l) = value.last_activity {
                    b.last_activity(l.seconds, l.nanos);
                }
                let u = b.build()?;
                Ok(u)
            }
        }
        impl From<shared::models::ListUsersResult> for ListUsersResponse {
            fn from(value: shared::models::ListUsersResult) -> Self {
                ListUsersResponse {
                    users: value.users.into_iter().map(User::from).collect(),
                    total_count: value.total_count,
                    current_page: value.current_page,
                    total_pages: value.total_pages,
                }
            }
        }
    }
    pub mod posts {

        tonic::include_proto!("proto.posts.v1");
        impl CreatePostRequest {
            pub fn convert(self, author_id: String) -> anyhow::Result<shared::models::Post> {
                let mut b = shared::models::Post::builder();
                let pdt = self
                    .publish_datetime
                    .as_ref()
                    .and_then(|d| chrono::DateTime::from_timestamp(d.seconds, d.nanos as u32));
                b.try_author_id(author_id)?
                    .title(self.title)
                    .content(self.content)
                    .tg_photo_file_id(self.tg_photo_file_id)
                    .vk_photo_file_id(self.vk_photo_file_id)
                    .tg_video_file_id(self.tg_video_file_id)
                    .vk_video_file_id(self.vk_video_file_id)
                    .publish_datetime(pdt);
                let p = b.build()?;
                Ok(p)
            }
        }
        impl From<shared::models::Post> for Post {
            fn from(value: shared::models::Post) -> Self {
                let sc: std::time::SystemTime = value.created_at.into();
                let pc = Some(sc.into());
                let sp: Option<std::time::SystemTime> = value.publish_datetime.map(|d| d.into());
                let pp = sp.map(|d| d.into());
                Post {
                    id: value.id.to_string(),
                    title: value.title,
                    content: value.content,
                    tg_photo_file_id: value.tg_photo_file_id,
                    vk_photo_file_id: value.vk_photo_file_id,
                    tg_video_file_id: value.tg_video_file_id,
                    vk_video_file_id: value.vk_video_file_id,
                    status: value.status.into(),
                    created_at: pc,
                    publish_datetime: pp,
                    author_id: value.author_id.to_string(),
                }
            }
        }
        impl TryFrom<Post> for shared::models::Post {
            type Error = anyhow::Error;
            fn try_from(value: Post) -> Result<Self, Self::Error> {
                let mut b = shared::models::Post::builder();
                let created = value
                    .created_at
                    .as_ref()
                    .and_then(|d| chrono::DateTime::from_timestamp(d.seconds, d.nanos as u32))
                    .unwrap_or(chrono::Utc::now());
                let pdt = value
                    .publish_datetime
                    .as_ref()
                    .and_then(|d| chrono::DateTime::from_timestamp(d.seconds, d.nanos as u32));
                b.try_id(value.id)?
                    .title(value.title)
                    .content(value.content)
                    .tg_photo_file_id(value.tg_photo_file_id)
                    .tg_video_file_id(value.tg_video_file_id)
                    .vk_photo_file_id(value.vk_photo_file_id)
                    .vk_video_file_id(value.vk_video_file_id)
                    .publish_datetime(pdt)
                    .created_at(created)
                    .try_status(value.status)?
                    .try_author_id(value.author_id)?;
                let p = b.build()?;
                Ok(p)
            }
        }
        impl From<shared::models::ListPostsResult> for ListPostsResponse {
            fn from(value: shared::models::ListPostsResult) -> Self {
                ListPostsResponse {
                    posts: value.posts.into_iter().map(Post::from).collect(),
                    total_count: value.total_count,
                    current_page: value.current_page,
                    total_pages: value.total_pages,
                }
            }
        }
    }
    pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("smm_descriptor");
}
