import type {encode as encodeType} from "@toon-format/toon";

let encodePromise: Promise<typeof encodeType> | undefined;
const importModule = new Function("specifier", "return import(specifier)") as (
    specifier: string,
) => Promise<typeof import("@toon-format/toon")>;

export async function encodeToon(value: unknown): Promise<string> {
    encodePromise ??= importModule("@toon-format/toon").then(module => module.encode);

    const encode = await encodePromise;

    return encode(value);
}
