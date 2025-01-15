use anyhow::Context as _;
use poise::serenity_prelude::{ClientBuilder, GatewayIntents};
use shuttle_runtime::SecretStore;
use shuttle_serenity::ShuttleSerenity;

const AVAILABLE_ROLES: &[&str] = &["baaaaaaaa", "boooo"];

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command)]
async fn hello(ctx: Context<'_>) -> Result<(), Error> {
    if let Some(roles) = ctx.guild().map(|g| g.roles.clone()) {
        if let Some(member) = ctx.author_member().await {
            for (role_id, role) in roles.iter() {
                if AVAILABLE_ROLES.contains(&role.name.as_str()) {
                    if member.roles.contains(role_id) {
                        member.remove_role(ctx, role_id).await?;
                        ctx.say(format!("you already had role {}", role.name))
                            .await?;
                    } else {
                        member.add_role(ctx, role_id).await?;
                        ctx.say(format!("role {} added", role.name)).await?;
                    }
                }
            }
        } else {
            ctx.say("world have no member").await?;
        }
    } else {
        ctx.say("world have no roles").await?;
    }

    Ok(())
}

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![hello()],
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
