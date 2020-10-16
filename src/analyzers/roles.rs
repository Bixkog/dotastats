use super::analyzers_utils::*;
use crate::constants::{Constants, Hero};
use crate::match_stats::Match;
use crate::match_stats::PlayerName;
use std::collections::HashMap;
use plotters::prelude::*;
pub type Roles = Vec<(PlayerName, String)>;

/// Finds heroes played by players.
fn get_heroes(
    constants: &Constants,
    match_: &Match,
    mut team: Vec<String>,
) -> Vec<(PlayerName, Hero)> {
    let mut team_setup = vec![];
    team.sort();
    for player in team {
        let player_hero_id = skip_fail!(match_.get_player_hero(&player));
        let hero = constants.get_hero(player_hero_id);
        team_setup.push((player, hero));
    }
    team_setup
}

/// As every hero can have several roles, this function generates subsets of those roles for a team.
/// e.g. P1: [A, B], P2: [B, C] -> ["P1-A P2-B", "P1-A P2-C", "P1-B P2-B", "P1-B P2-C"]
fn get_role_subsets(team_setup: Vec<(PlayerName, Hero)>) -> Vec<Roles> {
    let mut role_subsets = vec![Vec::new()];
    for (player, hero) in team_setup.iter() {
        role_subsets = [role_subsets.clone(), role_subsets
            .into_iter()
            .flat_map(|roles| {
                hero.roles
                    .iter()
                    .map(move |role| {
                        let mut roles_new = roles.clone();
                        roles_new.push((player.clone(), role.clone()));
                        roles_new
                    })
            })
            .collect()].concat()
    }
    role_subsets
}

pub fn roles_wr(data: &Vec<Match>) -> Vec<(Roles, WinRatio)> {
    // TODO: init earlier and pass
    let constants = Constants::init("heroes.txt".to_string());
    let mut roles_score: HashMap<Roles, WinRatio> = HashMap::new();
    for match_ in data {
        let team = skip_fail!(match_.get_team());
        let team_setup = get_heroes(&constants, match_, team);
        let role_subsets = get_role_subsets(team_setup);
        let is_won = skip_fail!(match_.is_won());
        for subset in role_subsets {
            if subset.is_empty() {continue}
            roles_score.entry(subset).or_default().add_score(is_won);
        }
    }
    let mut result: Vec<(Roles, WinRatio)> = roles_score.into_iter().collect();
    result = result
        .into_iter()
        .collect();
    result.sort_by_key(|(_, wr)| wr.clone());
    result.reverse();
    result
}

pub fn roles_synergies(data: &Vec<Match>) -> Vec<(Roles, (f64, u32))> {
    let roles_wr_res: Vec<(Roles, WinRatio)> = roles_wr(data).into_iter().filter(|(_, wr)| wr.total() > 30).collect();
    let single_wr = roles_wr_res.iter().filter(|(r, _)| r.len() == 1).fold(HashMap::<(String, String), WinRatio>::new(), 
        |mut s, (roles, wr)| {
            s.insert(roles[0].clone(), wr.clone());
            s
        } 
    );
    let mut result = vec![];
    for (roles, team_wr) in roles_wr_res {
        if roles.len() <= 1 {continue}
        let mut avg_solo_wr = 0.;
        for role in roles.iter() {
            avg_solo_wr += single_wr[role].as_percent();
        }
        if avg_solo_wr == 0. {continue}
        avg_solo_wr = avg_solo_wr / roles.len() as f64; 
        result.push((roles, (team_wr.as_percent() / avg_solo_wr, team_wr.total())));
    }
    result.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
    result
}

pub fn plot(res_roles_wr: &Vec<(Roles, WinRatio)>) -> Result<(), Box<dyn std::error::Error>> {
    let scale = res_roles_wr.iter().map(|(_, wr)| wr.wins.max(wr.looses)).max().unwrap() as f32 * 1.1;
    let single_results = res_roles_wr.iter().filter(|(r, _)| r.len() == 1).fold(HashMap::<String, Vec<(String, WinRatio)>>::new(), 
        |mut s, (roles, wr)| {
            let player_roles = s.entry(roles[0].0.clone()).or_default();
            player_roles.push((roles[0].1.clone(), wr.clone()));
            s
        } 
    );
    for (player, roles_wr) in single_results.iter() {
        println!("Player: {}", player);
        for (role, wr) in roles_wr.iter() {
            println!("  {} -> wr: {:.2} matches: {}", role, wr.as_percent(), wr.total());
        }
    }
    let root = BitMapBackend::new("0.png", (1920, 1080)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Roles winratio", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0f32..scale, 0f32..scale)?;

    chart.configure_mesh()
        .x_labels(20)
        .y_labels(10)
        .y_desc("Wins")
        .x_desc("Looses")
        .x_label_formatter(&|v| format!("{:.0}", v))
        .y_label_formatter(&|v| format!("{:.0}", v))
        .draw()?;
    
    chart.draw_series(LineSeries::new(
        vec![(0., 0.), (scale, scale)],
        &BLACK
    ))?;
    for (i, (player, roles_scores)) in single_results.into_iter().enumerate() {
        chart.draw_series(
    roles_scores.iter()
            .map(|(role, wr)| {
                    EmptyElement::at((wr.looses as f32, wr.wins as f32))
                    + Circle::new((0, 0), 5, PaletteColor::<Palette99>::pick(i).filled())
                    + Text::new(role.clone(),
                        (10, 0),
                        ("sans-serif", 10.0).into_font(),)
                }
            ))?
            .label(player.as_str())
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], PaletteColor::<Palette99>::pick(i).filled()));
    }
    chart.configure_series_labels().border_style(&BLACK).draw()?;
    Ok(())
}
