# Bale Proxy

[![Crates.io](https://img.shields.io/crates/v/bale-proxy)](https://crates.io/crates/bale-proxy)
[![Documentation](https://docs.rs/bale-proxy/badge.svg)](https://docs.rs/bale-proxy)
[![License](https://img.shields.io/crates/l/bale-proxy)](LICENSE)

A PoC secure channel on [Bale](https://bale.ai/) messenger to use as a plugin for [shadowsocks](https://crates.io/crates/shadowsocks-rust).

## Why?

Because of censorship that may happen one day in Iran
that blocks every type of traffic to outside world except
some government-backed messengers.

## How to run

1- Run server with plugin on a device in free world:
```bash
./ssserver --config conf.json --server-addr "127.0.0.1:8388" --encrypt-method "aes-256-gcm" --password "mypassword" --plugin /Path/To/bale-proxy --plugin-opts "server;log_level=debug;phone_number=4915207829731"
```

2- Run client with plugin on a device in censored network:
```bash
./sslocal --config conf.json --server-addr "127.0.0.1:8388" --local-addr "127.0.0.1:8387" --encrypt-method "aes-256-gcm" --password "mypassword" --plugin /Users/masihyeganeh/cargo-target/debug/bale-proxy --plugin-opts "client=932014429;log_level=debug;phone_number=4915207829731"
```

3- Set socks5 proxy in the client machine that points to `127.0.0.1:8387`.

4- Enjoy. (Kind of! It's just a PoC with millions of bugs)

### How to run without password prompts

Just login to [Bale](https://next.bale.ai/) and copy jwt from `token_bale` key in local storage and add it as a plugin option.

For example for server it can be:
```bash
... --plugin-opts "server;log_level=debug;phone_number=4915207829731;jwt=eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE2MzM3MjU4MDUxNTAsImlhdCI6MTYyODU0MTgwNTE1MCwiaXNzIjoiaHR0cHM6Ly93ZWIuYmFsZS5haSIsInBheWxvYWQiOnsiYXBwX2lkIjo0LCJhdXRoX2lkIjoiNTg1NzEzMDk2Mzg3NTE4MTQyNSIsImF1dGhfc2lkIjoxMTcxNDQ2MTgzLCJzZXJ2aWNlIjoid2ViX2xpdGUiLCJ1c2VyX2lkIjo5MzIwMTQ0Mjl9fQ.YCtOhcIVMBGjr2O0SpQM0fpQ_IWXUx2qsgvp5xCM0igx53ucalQ3vZI0YhYEqJSEW7eTIsFVUiW5mRhr2N-iw05T2x74XV2tMnjTZLPR4OdmG_r6eR1PGhcB9SnVZecuVD-Gs7Q1qld1D8gY46fc8yNRm9o4_zPGr__uJOdt8oK6DhZ-YMHCZlpXeFZgA35Nany66nSPIibN9N2B4iqQ_Lvk2x5a684LPvXcug0sTHZ7u7pHrvXFduhCLR4ekpf7ADfwts2bycRSEgoyefNMVnczFLoFPoioCNYRzYidKMz4_aR7KlSjWI0N6IuBHH7OauUIQBWfxdep45AF44e2RQAFF53n-daw38EdSkZg8Lq-1KBTQDDiN21jr38u4p_YJmpfQMc0Y3FFPjPJXzXITxsTSPxB5Vaf2Yd5PvI-BydAqL1OOGLct2vgSq0LlPZ_B-WRRqbTKo9xNv7wblZ50ufRWArpAbC999wz-2pRKQ78Ls7tSRfJ7ekF7U-LXX5-lWoINc63Cfkpyq-_n6PKkq4PWRCOwievbZrQSH6MBHul-m_3dbUwipFO-i07C11JqALRgZDzI4_dmEONMvdb5GDmeJWriZerPI5ClSnXECJLPzUS5GxrFgM_bCB7P6RqaALMdicpSvsI4SlVdznC_24fatVtyt7WnvN0ac2qSy8"
```

and for client it can be: 
```bash
... --plugin-opts "client=932014429;log_level=debug;phone_number=4915207829731;jwt=eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE2MzM1OTExODI1OTQsImlhdCI6MTYyODQwNzE4MjU5NCwiaXNzIjoiaHR0cHM6Ly93ZWIuYmFsZS5haSIsInBheWxvYWQiOnsiYXBwX2lkIjo0LCJhdXRoX2lkIjoiMTgxOTc2OTA4MTk4MzU4Njk3MCIsImF1dGhfc2lkIjo1ODE0NzYwMTYsInNlcnZpY2UiOiJ3ZWJfbGl0ZSIsInVzZXJfaWQiOjcwMzcyOTQ0N319.bliASxltkfu77u8XtFLq-4Vhk2RTLXPp0eYJDeL5f9Ag8YPz7QFTgJZBlwQiUCfh-50HiCQpOC27a6Z0giFqeGwN4H4boI3BR8dVc_9kjk-hoMnMogcWHmhISVq6oIQwAzeg_SWFrtA5Au0WTArLS6emQzoLyopUyo7kz_n3ONhIrgxTY5U-KY3dEnipLyTzXL91ctLi_2sX8PNq5mOS1QWjTpas05ZMCqCfRq4DkzOCXM_4CDyaR_Dm85LlhxO2Kk4gHnRAJBdOmpa1XJYUoSo_wmMAOfz1QM9bOmYhTUYgFBHOmuClYdGEtKBOfv6DbTnZs4C0EIm5-HP6T13ubeTXUJHHTclDKU1SZydPqoU_pX29zGZvPexZelfEU5VsNVIxvXz9KId2nMWJ9G_lzgFXSJmjYFH_7_ygKc29BA0eC6SkouJ22VexCUO5llSkuLPds-FfVsvbzzhArvcRdMZMoa8DZbHWZ0eggcbB8c6wtPP4-DlfgS7ZwtUQsPTVnd5nKUn5_wOfMeF2KNW4VWycz8dJ043pZzOiyuGgok3ddqhq9oMxuVc7MtYop58UpoMttUiHkCGlpFeIN7TvXqbHco0BXMEtEMF0eCp4RBJ00bMa0Wjzwlz2OxKXgz8VpVLuSmGpiPpckS_x2Wa93eR4Tk67G_XbIQFPn-0cuTg"
```

## And how does it look in the messenger?
![Just a bunch of base64 encoded texts](github/screenshot.png)  

## TODOs

- [ ] Get rid of async_std and just use Tokio
- [ ] Secure channel by handshaking
- [ ] Proper error handling
