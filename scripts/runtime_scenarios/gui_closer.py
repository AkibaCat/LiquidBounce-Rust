"""Physical inventory openings, prospective titles and actual language settings."""


def verify(*, api, control, check, settings, put_settings, leaf, enable, key,
           wait_status, wait_state, probe, fresh_probe, choose_language,
           exercised, report, run):
    control("release")
    enable("GUICloser", False)
    control("chat", "/gamemode survival")
    wait_state(lambda s: s.get("gamemode") == "survival")
    tree = settings("GUICloser")
    mode = leaf(tree, "Mode")
    filters = leaf(tree, "Filter")
    printing = leaf(tree, "PrintScreenTitle")
    samples = []

    def open_and_sample():
        before = api("/client/integration-status")
        key("e", "tap")
        status = wait_status(lambda s: fresh_probe(s, "clientGuiCloserScreen", before))
        return probe(status, "clientGuiCloserScreen")

    for language, expected_title in (("en_us", "Crafting"), ("zh_cn", "合成"), ("zh_tw", "合成")):
        choose_language(language)
        mode["value"] = "Matches"
        filters["value"] = ["^NeverMatchesThisInventory$"]
        printing["value"] = True
        put_settings("GUICloser", tree)
        enable("GUICloser", True)
        printed = open_and_sample()
        wait_state(lambda s: s.get("screen") == "inventory")
        check("guiCloserActualLocalizedTitleAndPrint" + language,
              printed.get("input", {}).get("title") == expected_title
              and printed.get("output", {}).get("printTitle") is True
              and printed["output"].get("cancel") is False, printed)
        key("e", "tap")
        wait_state(lambda s: s.get("screen") == "playing")
        control("screenshot", str(run / ("gui-closer-title-" + language + ".png")))
        filters["value"] = ["^" + expected_title + "$"]
        put_settings("GUICloser", tree)
        cancelled = open_and_sample()
        control("wait", "5")
        actual = control("state")
        check("guiCloserActualLocalizedOpeningCancelled" + language,
              actual.get("screen") == "playing"
              and cancelled.get("input", {}).get("title") == expected_title
              and cancelled.get("output", {}).get("cancel") is True
              and cancelled["output"].get("printTitle") is False, cancelled)
        samples.append({"language": language, "print": printed, "cancel": cancelled})
        enable("GUICloser", False)

    choose_language("en_us")
    filters["value"] = ["raft"]
    printing["value"] = False
    mode["value"] = "Matches"
    put_settings("GUICloser", tree)
    enable("GUICloser", True)
    matched = open_and_sample()
    wait_state(lambda s: s.get("screen") == "inventory")
    check("guiCloserMatchesRequiresEntireTitle", matched.get("output", {}).get("cancel") is False, matched)
    key("e", "tap")
    wait_state(lambda s: s.get("screen") == "playing")
    mode["value"] = "Contains"
    put_settings("GUICloser", tree)
    contained = open_and_sample()
    control("wait", "5")
    check("guiCloserContainsCancelsPhysicalInventoryOpen", control("state").get("screen") == "playing"
          and contained.get("output", {}).get("cancel") is True, contained)
    filters["value"] = [None, "^Crafting$"]
    mode["value"] = "Matches"
    printing["value"] = True
    put_settings("GUICloser", tree)
    aborted = open_and_sample()
    wait_state(lambda s: s.get("screen") == "inventory")
    check("guiCloserNullFilterAbortsBeforeMatchAndNotification",
          aborted.get("output", {}) == {"cancel": False, "printTitle": False, "nullFilter": True}, aborted)
    key("e", "tap")
    wait_state(lambda s: s.get("screen") == "playing")
    enable("GUICloser", False)
    control("chat", "/gamemode creative")
    wait_state(lambda s: s.get("gamemode") == "creative")
    filters["value"] = ["^"]
    mode["value"] = "Contains"
    printing["value"] = False
    put_settings("GUICloser", tree)
    enable("GUICloser", True)
    creative = open_and_sample()
    control("wait", "5")
    check("guiCloserCreativeEmptyTitleCancelsBeforeInitialization",
          creative.get("input", {}).get("title") == ""
          and creative.get("output", {}).get("cancel") is True
          and control("state").get("screen") == "playing", creative)
    enable("GUICloser", False)
    restored = open_and_sample()
    wait_state(lambda s: s.get("screen") == "creative")
    check("guiCloserDisableRestoresPhysicalInventoryOpen", restored.get("output", {}).get("cancel") is False, restored)
    key("e", "tap")
    wait_state(lambda s: s.get("screen") == "playing")
    exercised("GUICloser")
    report["guiCloserRuntimeSamples"] = samples
    report["guiCloserRuntimeCoverage"] = "physical_inventory_open;prospective_screen_title;EN_SC_TC_actual_game_language;Matches_Contains;null_filter_exception_capture;creative_empty_title;cancellation_and_disable_restore;other_entry_lifecycle_and_chat_styles_have_independent_native_fixtures"
    report["guiCloserScreenshotPendingVisualReview"] = [
        str(run / ("gui-closer-title-" + language + ".png")) for language in ("en_us", "zh_cn", "zh_tw")]
