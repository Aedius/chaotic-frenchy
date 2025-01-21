use anyhow::{anyhow, Context as _};
use poise::serenity_prelude::{
    Channel, ClientBuilder, Context, FullEvent, GatewayIntents, ReactionType,
};
use shuttle_runtime::SecretStore;
use shuttle_serenity::ShuttleSerenity;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = ClientBuilder::new(discord_token, GatewayIntents::non_privileged())
        .framework(framework)
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(client.into())
}

async fn event_handler(
    ctx: &Context,
    event: &FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::ReactionAdd {
            add_reaction: reaction,
        } => {
            if !reaction.message(ctx).await?.content.contains("votre rôle") {
                return Ok(());
            }

            println!(
                "reaction {:?} added by {:?}",
                reaction.clone().emoji,
                reaction.clone().member.map(|m| m.roles),
            );
            if let ReactionType::Custom {
                name: Some(custom_emoji),
                ..
            } = reaction.clone().emoji
            {
                if let Channel::Guild(g) = reaction.channel(ctx).await? {
                    let roles = g.guild(ctx).ok_or(anyhow!("no guild"))?.roles.clone();
                    for (role_id, role) in roles.iter() {
                        if role.name == custom_emoji {
                            if let Some(member) = reaction.clone().member {
                                member.add_role(ctx, role_id).await?
                            }
                        }
                    }
                }
            }
        }
        FullEvent::ReactionRemove {
            removed_reaction: reaction,
        } => {
            if !reaction.message(ctx).await?.content.contains("votre rôle") {
                return Ok(());
            }
            println!("reaction {:?} removed ", reaction.clone().emoji);

            if let (Some(guild_id), Some(user_id)) = (reaction.guild_id, reaction.user_id) {
                println!("reaction {:?}", reaction);
                if let ReactionType::Custom {
                    name: Some(custom_emoji),
                    ..
                } = reaction.clone().emoji
                {
                    let member = ctx.http.get_member(guild_id, user_id).await?;
                    if let Channel::Guild(g) = reaction.channel(ctx).await? {
                        let roles = g.guild(ctx).ok_or(anyhow!("no guild"))?.roles.clone();
                        for (role_id, role) in roles.iter() {
                            if role.name == custom_emoji {
                                member.remove_role(ctx, role_id).await?
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}
