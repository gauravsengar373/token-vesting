# Simple JS binding

## Quickstart

Run `yarn` in the `js` directory to install the node modules. Run `yarn dev` to get started and `yarn build` to build.

Contract address on Devnet

```
FhNbZ87io3fvp3f9Q3ggeJLQhNTEPCsuoW8SS1xJYzeN

and on main-net
3XYhmSPuXo2XyiAxypCBbg2CFiDXVPEPvbey8U2bgQ1c
```

See on the [Solana Explorer](https://explorer.solana.com/address/Hj9R6bEfrULLNrApMsKCEaHR9QJ2JgRtM381xgYcjFmQ?cluster=devnet)

The code allows you to

- Create vesting instructions for any SPL token: `createCreateInstruction`
- Create unlock instructions: `createUnlockInstruction`
- Change the destination of the vested tokens: `createChangeDestinationInstruction`

(To import Solana accounts created with [Sollet](https://sollet.io) you can use `getAccountFromSeed`)


