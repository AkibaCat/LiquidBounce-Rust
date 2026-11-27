// SPDX-License-Identifier: GPL-3.0-or-later
import {derived, writable} from "svelte/store";
import {REST_BASE, isStatic} from "./host";
import {listenAlways} from "./ws";
import {
    choiceLabel, description, englishSnapshot, label, moduleLabel, moduleSearchNames, settingDescription,
    type LocalizationSnapshot
} from "./localization-core";

// The server follows Horizon's actually loaded language. Never translate the
// objects returned by getModules/getModuleSettings: those are persistence data.
export const localization = writable<LocalizationSnapshot>(englishSnapshot);
export const translate = derived(localization, snapshot =>
    (name: string | null | undefined, spaced = false) => label(snapshot, name, spaced));
export const translateChoice = derived(localization, snapshot =>
    (setting: {key?: string}, name: string, spaced = false) => choiceLabel(snapshot, setting, name, spaced));
export const translateModule = derived(localization, snapshot =>
    (name: string, spaced = false) => moduleLabel(snapshot, name, spaced));
export const translateDescription = derived(localization, snapshot =>
    (original: string | undefined, moduleName?: string) => description(snapshot, original, moduleName));
export const translateSettingDescription = derived(localization, snapshot =>
    (setting: {name: string; description?: string; key?: string}) => settingDescription(snapshot, setting));
export const searchableModuleNames = derived(localization, snapshot =>
    (name: string, aliases: string[]) => moduleSearchNames(snapshot, name, aliases));

let requestRevision = 0;

async function refreshLocalization() {
    if (isStatic) return;
    const revision = ++requestRevision;
    try {
        const response = await fetch(`${REST_BASE}/api/v1/client/localization`);
        if (!response.ok) throw new Error(`Localization request returned ${response.status}`);
        const snapshot: LocalizationSnapshot = await response.json();
        if (revision !== requestRevision) return;
        localization.set(snapshot);
        document.documentElement.lang = snapshot.locale.replace("_", "-");
        document.documentElement.dataset.gameLanguage = snapshot.gameLanguage;
    } catch (error) {
        // Keep the last complete snapshot during a transport interruption.
        // socketReady refreshes it after reconnection.
        console.warn("[Localization] Could not refresh game language", error);
    }
}

listenAlways("socketReady", refreshLocalization);
listenAlways("languageChange", refreshLocalization);
refreshLocalization();
