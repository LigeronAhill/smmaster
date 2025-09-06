pub mod smm {
    pub mod users {
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
                b.try_id(value.id)?
                    .telegram_id(value.telegram_id)
                    .first_name(value.first_name)
                    .last_name(value.last_name)
                    .username(value.username)
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
    }
    pub mod posts {
        tonic::include_proto!("proto.posts.v1");
    }
    pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("smm_descriptor");
}
