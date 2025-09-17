# pacman-server

Despite the naming of this crate, it's not a server for the Pac-Man game allowing multiplayer or anything super interesting.

This crate is a webserver that hosts an OAuth login and leaderboard API for the main `pacman` crate to hook into.

## Features

- [x] Axum Webserver
- [x] Database
- [x] OAuth
  - [x] Discord
  - [x] GitHub
  - [ ] Google (?)
- [ ] Leaderboard API
- [ ] Name Restrictions & Flagging
- [ ] Avatars
  - [ ] 8-bit Conversion
  - [ ] Storage?
- [ ] Common Server/Client Crate
- [ ] CI/CD & Tests

## Todo

1. Refresh Token Handling (Encryption, Expiration & Refresh Timings)
2. Refresh Token Background Job
3. S3 Storage for Avatars
4. Common Server/Client Crate, Basics
5. Crate-level Log Level Configuration
6. Span Tracing
7. Avatar Pixelization
8. Leaderboard API
9. React-based Frontend
10. Name Restrictions & Flagging
11. Simple CI/CD Checks & Tests
12. API Rate Limiting (outbound provider requests)
13. API Rate Limiting (inbound requests, by IP, by User)
14. Provider Circuit Breaker
15. Merge migration files
