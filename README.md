# Shiny Pancake

## Info
- Unit tests can be found under `src/tests.rs` and ran with `cargo test`.

## Assumptions
- Withdrawals can be disputed.
  - The language in the description of a dispute makes it seem like you can only dispute a deposit but to me it makes sense that any transaction can be disputed.
  - The way I'm thinking about dispute -> chargeback is basically just "undoing" the transaction. So in the case of deposit then dispute -> chargeback "un-deposits"
    and removes the funds from the account. In the case of withdrawals dispute -> chargeback should "un-widthdraw" i.e. return the funds to the account as available.
    Obviously resolving any dispute should just allow the original operation to proceed as normal and any disputed funds should remain in held until resolved/chargebacked.
- Locked accounts cannot deposit or withdraw but disputes & resolves can still be applied.
- Charging back a dispute removes the funds from held but does not unmark the transaction as disputed.
