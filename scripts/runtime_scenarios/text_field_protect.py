"""Actual chat EditBox rendering; typed input stays intact and only drawing changes."""


def verify(*, api, control, check, settings, put_settings, leaf, enable, key,
           wait_status, wait_state, probe, fresh_probe, exercised, report, run):
    control("release")
    enable("TextFieldProtect", False)
    tree = settings("TextFieldProtect")
    patterns = leaf(tree, "Patterns")
    key("t", "tap")
    wait_state(lambda state: state.get("screen") == "chat")

    def type_and_sample(value, masked):
        key("ctrl", "down")
        try:
            key("a", "tap")
        finally:
            key("ctrl", "up")
        control("type", value)
        actual = control("screen")
        check("textFieldActualInputUnchanged" + str(len(report.get("textFieldProtect", []))),
              actual.get("chat_input") == value, actual)
        # Require another real draw after the typed value is observable; an
        # earlier frame of the previous value can have the same mask decision.
        before = api("/client/integration-status")
        status = wait_status(lambda status: fresh_probe(status, "textFieldFormat", before)
                             and probe(status, "textFieldFormat").get("output", {}).get("masked") is masked)
        sample = probe(status, "textFieldFormat")
        report.setdefault("textFieldProtect", []).append(sample)
        return sample

    plain = type_and_sample("/login visible-example", False)
    check("textFieldDisabledOriginalFormatter", plain["output"]["renderedLength"] == len("/login visible-example"), plain)
    control("screenshot", str(run / "text-field-disabled.png"))
    patterns["value"] = ["^/register.*", "^/login.*", "^/email.*"]
    put_settings("TextFieldProtect", tree)
    enable("TextFieldProtect", True)
    masked = type_and_sample("/login masked-example", True)
    check("textFieldDefaultMatchesActualFormatterAndUsesDisplayPos",
          masked["input"]["firstCharacterIndex"] == 0
          and masked["output"]["renderedLength"] == 0, masked)
    control("screenshot", str(run / "text-field-default-masked.png"))
    nonmatch = type_and_sample("ordinary chat example", False)
    check("textFieldDefaultNonmatchRendersOriginal", nonmatch["output"]["renderedLength"] == len("ordinary chat example"), nonmatch)
    patterns["value"] = [".*"]
    put_settings("TextFieldProtect", tree)
    scrolled = type_and_sample("中文😀" * 50, True)
    check("textFieldScrolledUtf16DisplayPosControlsMaskLength",
          scrolled["input"]["firstCharacterIndex"] > 0
          and scrolled["output"]["renderedLength"] == scrolled["input"]["firstCharacterIndex"], scrolled)
    control("screenshot", str(run / "text-field-scrolled-masked.png"))
    enable("TextFieldProtect", False)
    restored = type_and_sample("/login restored-example", False)
    check("textFieldDisableRestoresActualRendering", restored["output"]["renderedLength"] == len("/login restored-example"), restored)
    key("esc", "tap")
    wait_state(lambda state: state.get("screen") == "playing")
    exercised("TextFieldProtect")
