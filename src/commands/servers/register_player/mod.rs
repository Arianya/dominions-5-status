use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::channel::Message;
use serenity::model::id::UserId;

use server::ServerConnection;
use model::{GameServerState, Player};
use model::enums::*;
use db::{DbConnection, DbConnectionKey};
use super::alias_from_arg_or_channel_name;

fn register_player_helper<C: ServerConnection>(
    user_id: UserId,
    arg_nation_name: &str,
    alias: &str,
    db_conn: &DbConnection,
    message: &Message,
) -> Result<(), CommandError> {
    let server = db_conn.game_for_alias(&alias).map_err(CommandError::from)?;

    match server.state {
        GameServerState::Lobby(lobby_state) => {
            let players_nations = db_conn.players_with_nations_for_game_alias(&alias)?;
            if players_nations.len() as i32 >= lobby_state.player_count {
                return Err(CommandError::from("lobby already full"));
            };

            let nations = NATIONS_BY_ID
                .iter()
                .filter(|&(&_id, &(name, era))| {
                    let lname: String = name.to_owned().to_lowercase();
                    era == lobby_state.era && lname.starts_with(&arg_nation_name)
                })
                .collect::<Vec<_>>();
            let nations_len = nations.len();
            if nations_len > 1 {
                return Err(CommandError::from(
                    format!("ambiguous nation name: {}", arg_nation_name),
                ));
            } else if nations_len < 1 {
                return Err(CommandError::from(
                    format!("could not find nation: {}", arg_nation_name),
                ));
            };
            let (&nation_id, &(nation_name, nation_era)) = nations[0];
            if players_nations
                .iter()
                .find(|&&(_, player_nation_id)| {
                    player_nation_id == nation_id as usize
                })
                .is_some()
            {
                return Err(CommandError::from(
                    format!("Nation {} already exists in lobby", nation_name),
                ));
            }
            let player = Player {
                discord_user_id: user_id,
                turn_notifications: true,
            };
            // TODO: transaction
            db_conn.insert_player(&player).map_err(CommandError::from)?;
            db_conn
                .insert_server_player(&server.alias, &user_id, nation_id)
                .map_err(CommandError::from)?;
            message.reply(&format!(
                "registering {} {} for {}",
                nation_era,
                nation_name,
                user_id.get()?
            ))?;
            Ok(())
        }
        GameServerState::StartedState(started_state, _) => {
            let data = C::get_game_data(&started_state.address)?;

            // TODO: allow for players with registered nation but not ingame (not yet uploaded)
            let nations = data.nations
                .iter()
                .filter(|&nation| // TODO: more efficient algo
                nation.name.to_lowercase().starts_with(&arg_nation_name))
                .collect::<Vec<_>>();

            let nations_len = nations.len();
            if nations_len > 1 {
                return Err(CommandError::from(
                    format!("ambiguous nation name: {}", arg_nation_name),
                ));
            } else if nations_len < 1 {
                let error = if data.turn == -1 {
                    format!("Could not find nation starting with {}. Make sure you've uploaded a pretender first"
                            , arg_nation_name)
                } else {
                    format!("Could not find nation starting with {}", arg_nation_name)
                };
                return Err(CommandError::from(error));
            };

            let nation = nations[0];
            let player = Player {
                discord_user_id: user_id,
                turn_notifications: true,
            };

            // TODO: transaction
            db_conn.insert_player(&player).map_err(CommandError::from)?;
            info!("{} {} {}", server.alias, user_id, nation.id as u32);
            db_conn
                .insert_server_player(&server.alias, &user_id, nation.id as u32)
                .map_err(CommandError::from)?;
            let text = format!(
                "registering nation {} for user {}",
                nation.name,
                message.author
            );
            let _ = message.reply(&text);
            Ok(())
        }
    }
}

pub fn register_player<C: ServerConnection>(
    context: &mut Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let arg_nation_name: String = args.single_quoted::<String>()?.to_lowercase();
    let alias = alias_from_arg_or_channel_name(&mut args, &message)?;
// FIXME: no idea why this isn't working
//    if args.len() != 0 {
//        return Err(CommandError::from(
//            "Too many arguments. TIP: spaces in arguments need to be quoted \"like this\"",
//        ));
//    }

    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or("no db connection")?;

    register_player_helper::<C>(
        message.author.id,
        &arg_nation_name,
        &alias,
        db_conn,
        message,
    )?;
    Ok(())
}
