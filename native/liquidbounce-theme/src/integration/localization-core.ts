// SPDX-License-Identifier: GPL-3.0-or-later
// Display helpers have no mutation access to module/settings identifiers.
export interface LocalizationSnapshot {
    gameLanguage: string;
    locale: string;
    generation: number;
    moduleNames: Record<string, string>;
    labels: Record<string, string>;
    moduleKeys: Record<string, string>;
    translations: Record<string, string>;
}

export const englishSnapshot: LocalizationSnapshot = {
    gameLanguage: "en_us", locale: "en_us", generation: 0,
    moduleNames: {}, labels: {}, moduleKeys: {}, translations: {}
};

const wordBoundary = /[a-z](?=[A-Z0-9])|[A-Z0-9](?=[A-Z][a-z])/g;

function originalLabel(name: string, spaced: boolean): string {
    return spaced ? name.replace(wordBoundary, "$& ") : name;
}

export function label(snapshot: LocalizationSnapshot, name: string | null | undefined, spaced = false): string {
    if (name == null) return "";
    return snapshot.labels[name] ?? originalLabel(name, spaced);
}

// A setting's raw choice stays unchanged; its key disambiguates shared words.
export function choiceLabel(snapshot: LocalizationSnapshot, setting: {key?: string}, name: string, spaced = false): string {
    const key = setting.key && `${setting.key}.${name}`;
    return (key && snapshot.labels[key]) || label(snapshot, name, spaced);
}

export function moduleLabel(snapshot: LocalizationSnapshot, name: string, spaced = false): string {
    return snapshot.moduleNames[name] ?? originalLabel(name, spaced);
}

export function description(snapshot: LocalizationSnapshot, original: string | undefined, moduleName?: string): string {
    const key = moduleName && snapshot.moduleKeys[moduleName]
        ? `${snapshot.moduleKeys[moduleName]}.description`
        : original;
    return (key && snapshot.translations[key]) ?? original ?? "";
}

export function settingDescription(snapshot: LocalizationSnapshot, setting: {
    name: string; description?: string; key?: string
}): string {
    const explicitKey = setting.key && `${setting.key}.description`;
    const commonKey = (setting.name === "Bind" || setting.name === "Hidden")
        ? `liquidbounce.common.${setting.name.toLowerCase()}.description` : undefined;
    return (explicitKey && snapshot.translations[explicitKey])
        || (commonKey && snapshot.translations[commonKey])
        || description(snapshot, setting.description);
}

export function searchTerm(value: string): string {
    return value.toLocaleLowerCase().replace(/\s+/g, "");
}

export function moduleSearchNames(snapshot: LocalizationSnapshot, name: string, aliases: string[]): string[] {
    return [name, moduleLabel(snapshot, name), ...aliases].map(searchTerm);
}
