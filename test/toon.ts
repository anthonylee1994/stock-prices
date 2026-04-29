import type {encode as encodeType} from "@toon-format/toon";

export const encode: typeof encodeType = jest.fn((value: unknown) => JSON.stringify(value));
