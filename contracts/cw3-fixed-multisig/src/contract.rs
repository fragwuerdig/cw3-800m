use classic_bindings::{TerraQuery, TerraQuerier};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, BlockInfo, Deps, DepsMut, Empty, Env, MessageInfo, Order,
    Response, StdResult, StdError, Decimal, BankMsg, Coin,
};

use cw3::{
    Proposal, ProposalListResponse, ProposalResponse, VoteInfo,
    VoteListResponse, VoteResponse, VoterDetail, VoterListResponse, VoterResponse,
};
use cw_storage_plus::Bound;
use cw_utils::ThresholdResponse;

use crate::error::ContractError;
use crate::msg::QueryMsg;
use crate::state::{BALLOTS, CONFIG, PROPOSALS, VOTERS};

const BURN_ADDR: &str = "terra1sk06e3dyexuq4shw77y3dsv480xv42mq73anxu";

pub fn do_burn(
    deps: DepsMut<TerraQuery>,
    env: Env,
    where_to: String,
) -> StdResult<Response> {

    // ustc balance
    let my_address = env.contract.address;
    let uusd_balance_coin = deps.querier
        .query_balance(my_address, "uusd")?
        .amount;
    let uusd_balance = Decimal::from_atomics(uusd_balance_coin, 0)
        .map_err(|_| StdError::GenericErr { msg: String::from("invalid uusd balance") })?;
op
    // tax
    let querier = TerraQuerier::new(&deps.querier);
    let tax_rate = querier.query_tax_rate()?.rate; 
    let tax = tax_rate.checked_mul(uusd_balance)?;
    let to_send = uusd_balance.checked_sub(tax)?.to_uint_floor();
    let to_send_coin = Coin::new(
        to_send.u128(), 
        String::from("uusd")
    );

    // bank message - send uusd amount to burn address
    let message = BankMsg::Send {
        to_address: where_to,
        amount: vec![to_send_coin]
    };

    // emit the bank message
    Ok(Response::new()
        .add_message(message)
    )

}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut<TerraQuery>,
    env: Env,
    _msg: Empty,
) -> Result<Response, ContractError> {

    // in the migration handler try to burn contract uusd balance
    // if that fails try in exection entrypoint
    Ok(do_burn(deps, env, String::from(BURN_ADDR))?)

}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> Result<Response, ContractError> {

    // invalidate instantiation handler
    Err(ContractError::Std(StdError::GenericErr {
        msg: String::from("Do not instantiate this code!")
    }))

}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<TerraQuery>,
    env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> Result<Response<Empty>, ContractError> {

    // instead of cw3 execution burn the uusd contract balance
    Ok(do_burn(deps, env, String::from(BURN_ADDR))?)

}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    env: Env,
    msg: QueryMsg
) -> StdResult<Binary> {

    // maintain the query endpoints as normal
    match msg {
        QueryMsg::Threshold {} => to_binary(&query_threshold(deps)?),
        QueryMsg::Proposal { proposal_id } => to_binary(&query_proposal(deps, env, proposal_id)?),
        QueryMsg::Vote { proposal_id, voter } => to_binary(&query_vote(deps, proposal_id, voter)?),
        QueryMsg::ListProposals { start_after, limit } => {
            to_binary(&list_proposals(deps, env, start_after, limit)?)
        }
        QueryMsg::ReverseProposals {
            start_before,
            limit,
        } => to_binary(&reverse_proposals(deps, env, start_before, limit)?),
        QueryMsg::ListVotes {
            proposal_id,
            start_after,
            limit,
        } => to_binary(&list_votes(deps, proposal_id, start_after, limit)?),
        QueryMsg::Voter { address } => to_binary(&query_voter(deps, address)?),
        QueryMsg::ListVoters { start_after, limit } => {
            to_binary(&list_voters(deps, start_after, limit)?)
        }
    }

}

fn query_threshold(deps: Deps) -> StdResult<ThresholdResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(ThresholdResponse::AbsoluteCount {
        weight: cfg.required_weight,
        total_weight: cfg.total_weight,
    })
}

fn query_proposal(deps: Deps, env: Env, id: u64) -> StdResult<ProposalResponse> {
    let prop = PROPOSALS.load(deps.storage, id)?;
    let status = prop.current_status(&env.block);
    let threshold = prop.threshold.to_response(prop.total_weight);
    Ok(ProposalResponse {
        id,
        title: prop.title,
        description: prop.description,
        msgs: prop.msgs,
        status,
        expires: prop.expires,
        deposit: prop.deposit,
        proposer: prop.proposer,
        threshold,
    })
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

fn list_proposals(
    deps: Deps,
    env: Env,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<ProposalListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);
    let proposals = PROPOSALS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|p| map_proposal(&env.block, p))
        .collect::<StdResult<_>>()?;

    Ok(ProposalListResponse { proposals })
}

fn reverse_proposals(
    deps: Deps,
    env: Env,
    start_before: Option<u64>,
    limit: Option<u32>,
) -> StdResult<ProposalListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let end = start_before.map(Bound::exclusive);
    let props: StdResult<Vec<_>> = PROPOSALS
        .range(deps.storage, None, end, Order::Descending)
        .take(limit)
        .map(|p| map_proposal(&env.block, p))
        .collect();

    Ok(ProposalListResponse { proposals: props? })
}

fn map_proposal(
    block: &BlockInfo,
    item: StdResult<(u64, Proposal)>,
) -> StdResult<ProposalResponse> {
    item.map(|(id, prop)| {
        let status = prop.current_status(block);
        let threshold = prop.threshold.to_response(prop.total_weight);
        ProposalResponse {
            id,
            title: prop.title,
            description: prop.description,
            msgs: prop.msgs,
            status,
            deposit: prop.deposit,
            proposer: prop.proposer,
            expires: prop.expires,
            threshold,
        }
    })
}

fn query_vote(deps: Deps, proposal_id: u64, voter: String) -> StdResult<VoteResponse> {
    let voter = deps.api.addr_validate(&voter)?;
    let ballot = BALLOTS.may_load(deps.storage, (proposal_id, &voter))?;
    let vote = ballot.map(|b| VoteInfo {
        proposal_id,
        voter: voter.into(),
        vote: b.vote,
        weight: b.weight,
    });
    Ok(VoteResponse { vote })
}

fn list_votes(
    deps: Deps,
    proposal_id: u64,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<VoteListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let votes = BALLOTS
        .prefix(proposal_id)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            item.map(|(addr, ballot)| VoteInfo {
                proposal_id,
                voter: addr.into(),
                vote: ballot.vote,
                weight: ballot.weight,
            })
        })
        .collect::<StdResult<_>>()?;

    Ok(VoteListResponse { votes })
}

fn query_voter(deps: Deps, voter: String) -> StdResult<VoterResponse> {
    let voter = deps.api.addr_validate(&voter)?;
    let weight = VOTERS.may_load(deps.storage, &voter)?;
    Ok(VoterResponse { weight })
}

fn list_voters(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<VoterListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let voters = VOTERS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            item.map(|(addr, weight)| VoterDetail {
                addr: addr.into(),
                weight,
            })
        })
        .collect::<StdResult<_>>()?;

    Ok(VoterListResponse { voters })
}
