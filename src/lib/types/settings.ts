/**
 * Settings type system.
 *
 * Each `SettingEntry` pairs a key from `AppSettings` with a definition that
 * describes how the setting is displayed and edited. The discriminant `kind`
 * drives which input component is rendered in `SettingRow`.
 *
 * ## Adding a new setting
 *
 * 1. Add the field to `AppSettings` in `rss.ts` (and `AppSettingsRecord` in
 *    `src-tauri/src/models.rs` with a matching `schema.rs` migration).
 * 2. Add a `SettingEntry` to `APP_SETTINGS` in `config/settings.ts`.
 *    TypeScript enforces that `key` and `kind` are compatible.
 *
 * ## Adding a new kind
 *
 * 1. Define a new `*Def` interface below (must include `kind: '<name>'`).
 * 2. Add the corresponding `*Entry` type.
 * 3. Add it to the `SettingEntry` union.
 * 4. Create the matching input component in `components/settings/inputs/`.
 * 5. Add the dispatch branch in `SettingRow.svelte`.
 */

export type ColorScheme = 'system' | 'light' | 'dark';

export const DEFAULT_MAX_AUDIO_CACHE_SIZE_BYTES = 5 * 1024 * 1024 * 1024;
export const DEFAULT_MINI_PLAYER_ALWAYS_ON_TOP = false;
export const DEFAULT_AUTO_REFRESH_INTERVAL_MINUTES = 60;
export const DEFAULT_COLOR_SCHEME: ColorScheme = 'system';
export const DEFAULT_ACCENT_COLOR: string | null = null;
export const DEFAULT_SKIP_FORWARD_SECONDS = 15;
export const DEFAULT_SKIP_BACKWARD_SECONDS = 15;

export interface AppSettings {
	maxAudioCacheSizeBytes: number;
	miniPlayerAlwaysOnTop: boolean;
	autoRefreshIntervalMinutes: number;
	colorScheme: ColorScheme;
	accentColor: string | null;
	skipForwardSeconds: number;
	skipBackwardSeconds: number;
}

/** Keys of `Obj` whose values extend `T`. */
type KeysOfType<Obj, T> = { [K in keyof Obj]: Obj[K] extends T ? K : never }[keyof Obj];

/** AppSettings keys whose values are `boolean`. */
type BooleanKeys = KeysOfType<AppSettings, boolean>;

/** AppSettings keys whose values are `number`. */
type NumberKeys = KeysOfType<AppSettings, number>;

/**
 * AppSettings keys whose values are a non-nullable string (or string literal
 * union). These are valid targets for `'segmented'` entries.
 */
type StringKeys = {
	[K in keyof AppSettings]: AppSettings[K] extends string
		? null extends AppSettings[K]
			? never
			: K
		: never;
}[keyof AppSettings];

/**
 * AppSettings keys whose values are `string | null` — a nullable color or
 * string override. These are valid targets for `'color'` entries.
 */
type NullableStringKeys = {
	[K in keyof AppSettings]: null extends AppSettings[K]
		? Exclude<AppSettings[K], null> extends string
			? K
			: never
		: never;
}[keyof AppSettings];

/** Display metadata common to every setting kind. */
interface SettingMeta {
	/** Short title shown as the setting heading. */
	readonly label: string;
	/** One- or two-sentence explanation shown beneath the label. */
	readonly description: string;
	/** Brief note shown below the input (unit, timing, caveat, etc.). */
	readonly hint?: string;
	/**
	 * When `true` the input is disabled outside a Tauri runtime and a
	 * "desktop only" notice appears in the description column.
	 */
	readonly desktopOnly?: boolean;
}

/** A boolean toggle — rendered as a styled checkbox row. */
export interface ToggleDef extends SettingMeta {
	readonly kind: 'toggle';
	/**
	 * Primary label on the checkbox itself.
	 * Falls back to `label` when omitted.
	 */
	readonly checkboxLabel?: string;
}

/** A single option in a `SelectDef` or `SegmentedDef`. */
export interface SelectOption<V extends string | number> {
	readonly value: V;
	readonly label: string;
}

/** A dropdown `<select>` — suited to numeric settings with a fixed option list. */
export interface SelectDef<V extends string | number> extends SettingMeta {
	readonly kind: 'select';
	readonly options: ReadonlyArray<SelectOption<V>>;
}

/**
 * A segmented button group — suited to string settings with a small, fixed
 * option set (e.g. `'system' | 'light' | 'dark'`, `'compact' | 'comfortable'`).
 * Reuse this for any enum-like string setting; do not create domain-specific kinds.
 */
export interface SegmentedDef<V extends string> extends SettingMeta {
	readonly kind: 'segmented';
	readonly options: ReadonlyArray<SelectOption<V>>;
}

/** A numeric text input with optional display-unit conversion and inline validation. */
export interface NumberDef extends SettingMeta {
	readonly kind: 'number';
	readonly min?: number;
	readonly max?: number;
	readonly step?: number;
	/** Unit label appended after the input (e.g. `'GB'`). */
	readonly unit?: string;
	/**
	 * Convert the stored value to the value shown in the input.
	 * Example — bytes to GB: `(bytes) => bytes / BYTES_PER_GB`.
	 */
	readonly toDisplay?: (stored: number) => number;
	/**
	 * Convert the display value back to the stored value.
	 * Example — GB to bytes: `(gb) => Math.round(gb * BYTES_PER_GB)`.
	 */
	readonly fromDisplay?: (display: number) => number;
	/**
	 * Returns an error string for an invalid display value, or `null` when valid.
	 * Validation runs against the display value (after `toDisplay` is applied).
	 */
	readonly validate?: (display: number) => string | null;
}

/**
 * A nullable color picker — an enable/disable toggle paired with a hex
 * `<input type="color">`. Reuse this for any `string | null` color field;
 * do not create field-specific kinds.
 *
 * The stored value is `null` when the override is disabled (i.e. "use the
 * theme default") and a CSS hex string (e.g. `'#4f46e5'`) when active.
 */
export interface ColorDef extends SettingMeta {
	readonly kind: 'color';
	/** Hex value shown in the picker while no override is set (value is `null`). */
	readonly fallbackHex: string;
}

/** A boolean setting. `key` must be a `boolean` field in AppSettings. */
type BooleanEntry = { readonly key: BooleanKeys } & ToggleDef;

/** A numeric setting. `key` must be a `number` field in AppSettings. */
type NumericEntry = { readonly key: NumberKeys } & (NumberDef | SelectDef<number>);

/**
 * A segmented setting. `key` must be a non-nullable string field in AppSettings.
 * The distributed mapped type ensures each key's options are typed to its
 * specific value type (e.g. `ColorScheme` options for the `colorScheme` key).
 */
type SegmentedEntry = {
	[K in StringKeys]: { readonly key: K } & SegmentedDef<Extract<AppSettings[K], string>>;
}[StringKeys];

/** A nullable color setting. `key` must be a `string | null` field in AppSettings. */
type ColorEntry = { readonly key: NullableStringKeys } & ColorDef;

/**
 * A single row in the settings UI.
 *
 * The `kind` discriminant determines which input component is rendered.
 * The `key` binds the entry to its field in `AppSettings`.
 *
 * | kind        | renders as              | valid key types      |
 * |-------------|-------------------------|----------------------|
 * | `toggle`    | checkbox row            | `boolean` fields     |
 * | `number`    | numeric text input      | `number` fields      |
 * | `select`    | `<select>` dropdown     | `number` fields      |
 * | `segmented` | button group            | non-nullable `string` fields |
 * | `color`     | nullable color picker   | `string \| null` fields |
 */
export type SettingEntry = BooleanEntry | NumericEntry | SegmentedEntry | ColorEntry;

/** Ordered array of setting entries that drives the settings UI. */
export type SettingsDefinition = ReadonlyArray<SettingEntry>;
