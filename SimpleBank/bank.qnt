module bank {
  import basicSpells.* from "./spells/basicSpells"

  type Investment = { owner: str, amount: int }

  type BankState = {
    balances: str -> int,
    investments: int -> Investment,
    next_id: int
  }

  type BankResult = Ok(BankState) | Err(str)

  var bank_state: BankState
  var log: List[BankState]
  var error: Option[str]

  // forced prefix of actions to take
  var forced: List[Action]

  pure def increaseBalance(bs: str -> int, key: str, amount: int): str -> int =
    if (bs.keys().contains(key)) {
      bs.setBy(key, current => current + amount)
    }
    else {
      bs.put(key, amount)
    }

  pure def deposit(bank_state: BankState, depositor: str, amount: int): BankResult = {
    if (amount <= 0) {
      Err("Amount should be greater than zero")
    } else {
      val new_bank_state = {
        ...bank_state,
        balances: bank_state.balances.increaseBalance(depositor, amount)
      }
      Ok(new_bank_state)
    }
  }

  pure def withdraw(bank_state: BankState, withdrawer: str, amount: int): BankResult = {
    if (amount <= 0) {
      Err("Amount should be greater than zero")
    } else if (not(bank_state.balances.keys().contains(withdrawer))) {
      Err("Withdrawer does not have an account")
    }
    else if (bank_state.balances.get(withdrawer) < amount) {
      Err("Balance is too low")
    } else {
      val new_bank_state = {
        ...bank_state,
        balances: bank_state.balances
          .setBy(withdrawer, current => current - amount)
      }
      Ok(new_bank_state)
    }
  }

  pure def transfer(bank_state: BankState, sender: str, receiver: str, amount: int): BankResult = {
    if (amount <= 0) {
      Err("Amount should be greater than zero")
    } else if (not(bank_state.balances.keys().contains(sender))) {
      Err("Sender does not have an account")
    } else if (bank_state.balances.get(sender) < amount) {
      Err("Balance is too low")
    } else {
      val new_bank_state = {
        ...bank_state,
        balances: bank_state.balances
          .setBy(sender, current => current - amount)
          .increaseBalance(receiver, amount)
      }
      Ok(new_bank_state)
    }
  }

  pure def buy_investment(bank_state: BankState, buyer: str, amount: int): BankResult = {
    if (amount <= 0) {
      Err("Amount should be greater than zero")
    } else if (not(bank_state.balances.keys().contains(buyer))) {
      Err("Buyer does not have an account")
    } else if (bank_state.balances.get(buyer) < amount) {
      Err("Balance is too low")
    } else {
      val new_bank_state = {
        next_id: bank_state.next_id + 1,
        investments: bank_state.investments.put(bank_state.next_id, { owner: buyer, amount: amount }),
        balances: bank_state.balances
          .setBy(buyer, current => current - amount)
      }
      Ok(new_bank_state)
    }
  }

  pure def sell_investment(bank_state: BankState, seller: str, id: int): BankResult = {
    if (not(bank_state.investments.has(id))) {
      Err("No investment with this id")
    } else {
      pure val investment = bank_state.investments.get(id)
      if (investment.owner != seller) {
        Err("Seller can't sell an investment they don't own")
      } else {
        val new_bank_state = {
          ...bank_state,
          investments: bank_state.investments.mapRemove(id),
          balances: bank_state.balances
            .setBy(seller, current => current + investment.amount)
        }
        Ok(new_bank_state)
      }
    }
  }

  val ACCOUNTS = Set("alice", "bob")
  val AMOUNTS = (-50).to(100)

  action assign_result(r: BankResult): bool = match r {
    | Ok(state) => all {
        bank_state' = state,
        error' = None,
        log' = log.append(state),
      }
    | Err(err) => all {
        bank_state' = bank_state,
        error' = Some(err),
        log' = log,
      }
  }

  // Reified actions
  type Action
    = Deposit({ depositor: str, amount: int })
    | Withdraw({ withdrawer: str, amount: int })
    | Transfer({ sender: str, receiver: str, amount: int })
    | BuyInvestment({ buyer: str, amount: int })
    | SellInvestment({ seller: str, investment_id: int })

  action deposit_act(depositor: str, amount: int): bool =
    assign_result(bank_state.deposit(depositor, amount))
  action withdraw_act(withdrawer: str, amount: int): bool =
    assign_result(bank_state.withdraw(withdrawer, amount))
  action transfer_act(sender: str, receiver: str, amount: int): bool =
    assign_result(bank_state.transfer(sender, receiver, amount))
  action buy_investment_act(buyer: str, amount: int): bool =
    assign_result(bank_state.buy_investment(buyer, amount))
  action sell_investment_act(seller: str, id: int): bool =
    assign_result(bank_state.sell_investment(seller, id))

  // TODO: cleanup step function and non-det picks
  action deposit_action = {
    nondet amount    = AMOUNTS.oneOf()
    nondet depositor = ACCOUNTS.oneOf()
    deposit_act(depositor, amount)
  }

  action withdraw_action = {
    nondet amount     = AMOUNTS.oneOf()
    nondet withdrawer = ACCOUNTS.oneOf()
    withdraw_act(withdrawer, amount)
  }

  action transfer_action = {
    nondet amount   = AMOUNTS.oneOf()
    nondet sender   = ACCOUNTS.oneOf()
    nondet receiver = ACCOUNTS.oneOf()
    transfer_act(sender, receiver, amount)
  }

  action buy_investment_action = {
    nondet amount = AMOUNTS.oneOf()
    nondet buyer = ACCOUNTS.oneOf()
    buy_investment_act(buyer, amount)
  }

  action sell_investment_action = {
    nondet id = 0.to(10).oneOf()
    nondet seller = ACCOUNTS.oneOf()
    sell_investment_act(seller, id)
  }

  action initWithForced(actions: List[Action]): bool = {
    val initState = {
      balances: Map(),
      investments: Map(),
      next_id: 0,
    }
    all {
      bank_state' = initState,
      error'      = None,
      log'        = [initState],
      forced'     = actions,
    }
  }

  action init = initWithForced([])

  action force_step(act: Action): bool = {
    match act {
      | Deposit(p)        => deposit_act(p.depositor,  p.amount)
      | Withdraw(p)       => withdraw_act(p.withdrawer, p.amount)
      | Transfer(p)       => transfer_act(p.sender, p.receiver, p.amount)
      | BuyInvestment(p)  => buy_investment_act(p.buyer, p.amount)
      | SellInvestment(p) => sell_investment_act(p.seller, p.investment_id)
    }
  }

  action step =
    if (forced.length() > 0) all {
      force_step(forced.head()),
      forced' = forced.tail(),
    } 

    else all {
      forced' = forced,

      any {
        deposit_action,
        withdraw_action,
        transfer_action,
        buy_investment_action,
        sell_investment_action,
      }
    }
}
