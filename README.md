# ocean_cleanup

## Project Title
ocean_cleanup

## Project Description

Every year, millions of tons of plastic and debris end up in the ocean, but the
work done by volunteer cleanup crews is rarely recorded in a way that is
verifiable, transparent, and tamper-proof. The `ocean_cleanup` dApp is a
Soroban smart contract that lets a cleanup crew log an event (location hash,
kilograms collected, and a photo hash as proof), have a designated captain
verify it, and then automatically earn credit toward a global on-chain
leaderboard. Because every entry lives on Stellar, sponsors, NGOs, and the
public can audit who cleaned what, when, and where — turning good-faith
volunteer work into a verifiable on-chain record of impact.

## Project Vision

To build a transparent, decentralized registry of ocean-cleanup activity that
turns verified volunteer effort into a globally visible, portable credential.
By gamifying cleanup work through an on-chain leaderboard, the project aims to
incentivize participation, attract sponsors, and give cleanup organizations a
trustworthy way to demonstrate real-world environmental impact to funders and
the public — without relying on a single central operator.

## Key Features

- **Crew event logging** — `log_event` lets a crew member record a cleanup
  event on-chain with a `location_hash`, `kg_collected`, and `photo_hash` as
  cryptographic proof, all gated by `require_auth()`.
- **Captain verification** — `verify_event` is restricted to a single
  designated captain address that is set during `init`, preventing random
  accounts from approving their own work.
- **Two-step lifecycle** — every event moves through `Pending` → `Verified` →
  `Rewarded`, so double-counting is impossible and the reward step is
  explicit and auditable.
- **Per-crew and global leaderboards** — `get_crew_total` and
  `get_global_total` expose running totals that update only when `reward` is
  called on a verified event, making rankings fair and reproducible.
- **Status views** — `get_event_status`, `is_verified`, and `get_event` let
  any client (web UI, mobile app, sponsor dashboard) read the current state
  of any event or crew without sending a transaction.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** environment dApp — see `contracts/ocean_cleanup/src/lib.rs` for the full ocean_cleanup business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `CCBUX7WPTMPLTVDK2SIZYLQMWLIY45FNHONLRU476PZ66A75GNE62AHV`
- **Explorer template:** `https://stellar.expert/explorer/testnet/tx/c8338b3195b6d48e533b0dcea1566b995a37b4ca3d9072b044ee8c0203e48c54`

## Future Scope

- **Token rewards** — mint and distribute an `OCEAN` SAC token to crews on
  successful `reward`, turning verified impact into a tradable asset.
- **Multi-captain councils** — replace the single-captain model with a
  weighted quorum of verifiers (e.g. 2-of-3 regional captains) for
  decentralized trust.
- **Photo & location attestation** — integrate an off-chain oracle (e.g.
  IPFS + signed EXIF metadata) so `photo_hash` and `location_hash` can be
  cryptographically tied to a real geotagged image.
- **Time-bounded leaderboards** — monthly, quarterly, and all-time rankings
  with reset/reveal ceremonies to keep competitions fresh.
- **Frontend dApp** — a React + Freighter web UI for crews to log events
  with their phone camera and for sponsors to browse the leaderboard live.
- **Mainnet & governance** — graduate from Testnet to a governed Mainnet
  deployment with a Stellar DAO controlling the captain registry and the
  reward token treasury.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `ocean_cleanup` (environment)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
