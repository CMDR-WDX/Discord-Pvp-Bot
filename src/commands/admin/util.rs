use poise::serenity_prelude::{RoleId, GuildId};

use crate::{data::Environment, Context};

pub async fn is_user_admin(ctx: &Context<'_>) -> bool {
    let required_role_id = RoleId(Environment::role_admin());
    let required_guild_id = GuildId(Environment::guild_id());

    let user = ctx.author();

    return match user.has_role(ctx.http(), required_guild_id, required_role_id).await {
        Ok(is_admin) => is_admin,
        Err(err) => {println!("Failed to check if user {} has admin role. Reason: {}", user.id, err.to_string()); false},
    }

}