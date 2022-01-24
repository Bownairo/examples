use candid::{candid_method, export_service, CandidType, Nat, Principal};
use ic_cdk::{caller, println};
use ic_cdk_macros::*;
use ic_ledger_types::{
    AccountIdentifier, BlockIndex, Memo, Subaccount, Tokens, DEFAULT_SUBACCOUNT,
    MAINNET_LEDGER_CANISTER_ID,
};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct Conf {
    withdrawl_fee: f32,
    ledger_canister_id: Principal,
}

#[derive(CandidType)]
pub struct Balance {
    token_canister_id: Principal,
    amount: Nat,
}

#[derive(CandidType, Clone)]
pub struct Order {
    id: u64,
    owner: Principal,
    from_token_canister_id: Principal,
    from_amount: Nat,
    to_token_canister_id: Principal,
    to_amount: Nat,
}

#[derive(CandidType, Clone, Deserialize, Serialize, Copy)]
pub struct OrderState {
    id: u64,
    owner: Principal,
    from_token_canister_id: Principal,
    from_amount: u128,
    to_token_canister_id: Principal,
    to_amount: u128,
}

type OrdersState = HashMap<u64, OrderState>;
type BalancesState = HashMap<Principal, HashMap<Principal, u128>>; // owner -> token_canister_id -> amount

#[derive(CandidType, Clone, Deserialize, Serialize)]
pub struct State {
    conf: Conf,
    next_id: u64,
    balances: BalancesState,
    orders: OrdersState,
}

impl From<OrderState> for Order {
    fn from(o: OrderState) -> Order {
        Order {
            id: o.id,
            owner: o.owner,
            from_token_canister_id: o.from_token_canister_id,
            from_amount: o.from_amount.into(),
            to_token_canister_id: o.to_token_canister_id,
            to_amount: o.to_amount.into(),
        }
    }
}

export_service!();

#[ic_cdk_macros::query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    __export_service()
}

fn add_balance(m: &mut HashMap<Principal, u128>, token_canister_id: &Principal, delta: u128) {
    if let Some(x) = m.get_mut(&token_canister_id) {
        *x += delta;
    } else {
        m.insert(*token_canister_id, delta);
    }
}

fn subtract_balance(m: &mut HashMap<Principal, u128>, token_canister_id: &Principal, delta: u128) {
    let x = m.get_mut(&token_canister_id).unwrap();
    *x -= delta;
    if *x == 0 {
        m.remove(token_canister_id);
    }
}

impl Default for State {
    fn default() -> Self {
        State {
            conf: Conf {
                ledger_canister_id: MAINNET_LEDGER_CANISTER_ID,
                withdrawl_fee: 0.01,
            },
            next_id: 0,
            balances: BalancesState::new(),
            orders: OrdersState::new(),
        }
    }
}

impl State {
    fn get_balance(&self, token_canister_id: Principal) -> Option<Balance> {
        match self.balances.get(&caller()) {
            None => None,
            Some(v) => {
                if let Some(amount) = v.get(&token_canister_id) {
                    Some(Balance {
                        token_canister_id,
                        amount: (*amount).into(),
                    })
                } else {
                    None
                }
            }
        }
    }

    fn get_balances(&self) -> Option<Vec<Balance>> {
        match self.balances.get(&caller()) {
            None => None,
            Some(v) => Some(
                v.iter()
                    .map(|(token_canister_id, amount)| Balance {
                        token_canister_id: *token_canister_id,
                        amount: (*amount).into(),
                    })
                    .collect(),
            ),
        }
    }

    fn deposit(&mut self, token_canister_id: Principal, amount: Nat) -> String {
        let o = self.balances.get_mut(&caller()).unwrap();
        let from_balance = o.get_mut(&token_canister_id.clone()).unwrap();
        let amount: BigUint = amount.try_into().unwrap();
        let amount: u128 = amount.try_into().unwrap();
        *from_balance += amount;
        "ok".into()
        //let amount: BigUint = amount.try_into().unwrap();
        //let _amount: u128 = amount.try_into().unwrap();
        /*
        let canister_id = ic_cdk::api::id();
        let account = AccountIdentifier::new(&canister_id, Subaccount::from(caller()));
        println!(
            "Deposit of {} ICP in account {:?}",
            &amount, &token_canister_id, &account
        );
        let ledger_canister_id = CONF.with(|conf| conf.borrow().ledger_canister_id);
        let transfer_args = CONF.with(|conf| {
            let conf = conf.borrow();
            ic_ledger_types::TransferArgs {
                memo: Memo(0),
                amount: args.amount,
                fee: conf.transaction_fee,
                from_subaccount: conf.subaccount,
                to: AccountIdentifier::new(&args.to_principal, &to_subaccount),
                created_at_time: None,
            }
        });
        ic_ledger_types::transfer(ledger_canister_id, transfer_args)
            .await
            .map_err(|e| println!("failed to call ledger: {:?}", e))?
            .map_err(|e| println!("ledger transfer error {:?}", e))
            */
    }

    fn get_order(&self, order: u64) -> Option<Order> {
        match self.orders.get(&order) {
            None => None,
            Some(o) => Some((*o).into()),
        }
    }

    fn get_orders(&self) -> Vec<Order> {
        self.orders.iter().map(|(_, o)| (*o).into()).collect()
    }

    fn get_from_orders(&self, token_canister_id: Principal) -> Vec<Order> {
        self.orders
            .iter()
            .filter(|(_, o)| o.from_token_canister_id == token_canister_id)
            .map(|(_, o)| (*o).into())
            .collect()
    }

    fn get_to_orders(&self, token_canister_id: Principal) -> Vec<Order> {
        self.orders
            .iter()
            .filter(|(_, o)| o.to_token_canister_id == token_canister_id)
            .map(|(_, o)| (*o).into())
            .collect()
    }

    fn next_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    fn place_order(
        &mut self,
        from_token_canister_id: Principal,
        from_amount: Nat,
        to_token_canister_id: Principal,
        to_amount: Nat,
    ) -> String {
        let id = self.next_id();
        let from_amount: BigUint = from_amount.try_into().unwrap();
        let from_amount: u128 = from_amount.try_into().unwrap();
        let to_amount: BigUint = to_amount.try_into().unwrap();
        let balance = self.get_balance(from_token_canister_id);
        if let Some(b) = balance {
            if b.amount < from_amount {
                return "not enough from tokens".into();
            }
        } else {
            return "not enough from tokens".into();
        }
        self.orders.insert(
            id,
            OrderState {
                id,
                owner: caller(),
                from_token_canister_id,
                from_amount: from_amount,
                to_token_canister_id,
                to_amount: to_amount.try_into().unwrap(),
            },
        );
        self.resolve_order(id);
        "ok".into()
    }

    fn cancel_order(&mut self, order: u64) -> String {
        if let Some(o) = self.orders.get(&order) {
            if o.owner != caller() {
                return "not owner".into();
            }
            self.orders.remove(&order);
            "ok".into()
        } else {
            "no found".into()
        }
    }

    fn resolve_order(&mut self, id: u64) {
        let mut matches = Vec::new();
        {
            let a = self.orders.get(&id).unwrap();
            for (order, b) in self.orders.iter() {
                if a.from_token_canister_id == b.to_token_canister_id
                    && a.to_token_canister_id == b.from_token_canister_id
                {
                    let a_ratio = a.from_amount / a.to_amount;
                    let b_ratio = b.to_amount / b.from_amount;
                    if a_ratio == b_ratio {
                        matches.push((id, *order));
                    }
                }
            }
        }
        for m in matches {
            let mut amount = 0;
            {
                if let Some(a) = self.orders.get(&m.0) {
                    if let Some(b) = self.orders.get(&m.1) {
                        amount = std::cmp::min(a.from_amount, b.to_amount);
                    }
                }
            }
            if amount > 0 {
                self.process_trade(m, amount);
            }
        }
    }

    fn process_trade(&mut self, pair: (u64, u64), amount: u128) {
        let remove_a;
        let remove_b;
        let reverse_amount;
        {
            {
                // Update from side.
                let a = self.orders.get_mut(&pair.0).unwrap();
                reverse_amount = ((BigUint::from(amount) * BigUint::from(a.to_amount))
                    / BigUint::from(a.from_amount))
                .try_into()
                .unwrap();
                a.from_amount -= amount;
                remove_a = a.from_amount == 0;

                let o = self.balances.get_mut(&a.owner.clone()).unwrap();
                subtract_balance(o, &a.from_token_canister_id, amount);
                add_balance(o, &a.to_token_canister_id, reverse_amount);
            }

            {
                // Update to side.
                let b = self.orders.get_mut(&pair.1).unwrap();
                b.to_amount -= amount;
                remove_b = b.to_amount == 0;

                let o = self.balances.get_mut(&b.owner.clone()).unwrap();
                add_balance(o, &b.to_token_canister_id, amount);
                subtract_balance(o, &b.from_token_canister_id, reverse_amount);
            }
        }
        if remove_a {
            self.orders.remove(&pair.0);
        }
        if remove_b {
            self.orders.remove(&pair.1);
        }
    }
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

fn principal_to_subaccount(principal_id: &Principal) -> Subaccount {
    let mut subaccount = [0; std::mem::size_of::<Subaccount>()];
    let principal_id = principal_id.as_slice();
    subaccount[0] = principal_id.len().try_into().unwrap();
    subaccount[1..1 + principal_id.len()].copy_from_slice(principal_id);
    Subaccount(subaccount)
}

#[init]
#[candid_method(init)]
pub fn init(conf: Conf) -> () {
    ic_cdk::setup();
    println!("init!");
    STATE.with(|s| s.borrow_mut().conf = conf);
}

#[pre_upgrade]
fn pre_upgrade() {
    let stable_state = STATE.with(|s| s.take());
    ic_cdk::storage::stable_save((stable_state,)).expect("failed to save stable state");
}

#[post_upgrade]
fn post_upgrade() {
    let (stable_state,) =
        ic_cdk::storage::stable_restore().expect("failed to restore stable state");
    STATE.with(|s| {
        s.replace(stable_state);
    });
}

#[query]
#[candid_method(query)]
pub fn get_balance(token_canister_id: Principal) -> Option<Balance> {
    STATE.with(|s| s.borrow().get_balance(token_canister_id))
}

#[query]
#[candid_method(query)]
pub fn get_balances() -> Option<Vec<Balance>> {
    STATE.with(|s| s.borrow().get_balances())
}

#[update]
#[candid_method(update)]
pub fn deposit(token_canister_id: Principal, amount: Nat) -> String {
    STATE.with(|s| s.borrow_mut().deposit(token_canister_id, amount))
}

#[query]
#[candid_method(query)]
pub fn get_order(order: u64) -> Option<Order> {
    STATE.with(|s| s.borrow().get_order(order))
}

#[query]
#[candid_method(query)]
pub fn get_orders() -> Vec<Order> {
    STATE.with(|s| s.borrow().get_orders())
}

#[query]
#[candid_method(query)]
pub fn get_from_orders(token_canister_id: Principal) -> Vec<Order> {
    STATE.with(|s| s.borrow().get_from_orders(token_canister_id))
}

#[query]
#[candid_method(query)]
pub fn get_to_orders(token_canister_id: Principal) -> Vec<Order> {
    STATE.with(|s| s.borrow().get_to_orders(token_canister_id))
}

#[update]
#[candid_method(update)]
pub fn place_order(
    from_token_canister_id: Principal,
    from_amount: Nat,
    to_token_canister_id: Principal,
    to_amount: Nat,
) -> String {
    STATE.with(|s| {
        s.borrow_mut().place_order(
            from_token_canister_id,
            from_amount,
            to_token_canister_id,
            to_amount,
        )
    })
}

#[update]
#[candid_method(update)]
pub fn cancel_order(order: u64) -> String {
    STATE.with(|s| s.borrow_mut().cancel_order(order))
}

#[update]
#[candid_method(update)]
pub fn withdraw(_token_canister_id: Principal, _amount: Nat, _to_principal: Principal) -> () {}

#[query]
#[candid_method(query)]
pub fn whoami() -> Principal {
    caller()
}

#[query]
#[candid_method(query)]
pub fn icp_deposit_account() -> String {
    let canister_id = ic_cdk::api::id();
    let subaccount = principal_to_subaccount(&caller());
    let account = AccountIdentifier::new(&canister_id, &subaccount).to_string();
    println!("icp deposit account {}!", account);
    account
}
