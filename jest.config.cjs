module.exports = {
    collectCoverageFrom: ["src/stock-prices/*.ts", "!src/stock-prices/*.spec.ts", "!src/stock-prices/*.module.ts", "!src/stock-prices/*.type.ts"],
    moduleFileExtensions: ["js", "json", "ts"],
    rootDir: ".",
    testRegex: ".*\\.spec\\.ts$",
    transform: {
        "^.+\\.(t|j)s$": [
            "ts-jest",
            {
                tsconfig: {
                    emitDecoratorMetadata: false,
                    experimentalDecorators: true,
                    module: "nodenext",
                    moduleResolution: "nodenext",
                    target: "ES2023",
                },
            },
        ],
    },
    moduleNameMapper: {
        "^@toon-format/toon$": "<rootDir>/test/toon.ts",
    },
    testEnvironment: "node",
};
