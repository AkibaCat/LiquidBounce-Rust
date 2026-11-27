# AntiStaff integration

Already integrated through `integrate-anti-staff.py --apply`, with locale additions from `update-anti-staff-locales.py`. Do not reapply the transactional, non-idempotent integrator.

Preserves ShowInTabList, the exact case-sensitive staff membership check, a cross-server domain cache, empty reservation before requests, and no retry after an unavailable list. Address normalization follows the pinned `dropPort`/`rootDomain` implementation: remove valid ports, preserve IPv4 and IPv6, otherwise keep the last two domain components. It intentionally does not introduce a public suffix database absent from the source.

Enabling requests the current uncached server immediately. Connection changes reserve the domain then wait for an in-game screen, matching the source coroutine. Disabled generations discard late worker completions. Native HTTP uses the real ClientCdn staffs endpoint through the existing ureq TLS dependency on a worker thread. Successful bodies use Kotlin lineSequence semantics, including an empty trailing line. Actual 404, other HTTP and transport errors produce localized notifications. No fabricated list is used.

Original ADD_PLAYER entries trigger local notifications and metadata-replacing chat messages (`AntiStaff#name`). Tab rows append the soft-red Staff suffix, localized for Chinese; merging a staff policy does not require BetterTab to be enabled. The shared Tab policy cache observes staff and language changes. Server chat is never sent by this module.

One focused smoke passed: `build/anti-staff-smoke-01.log`. It covers domain/IP normalization, empty cache reservation, no retry, case sensitivity, tab toggle and connection wait. Official native plus engine Check157-01 is pending at handoff. No live CDN/gameplay or full module test is claimed.
