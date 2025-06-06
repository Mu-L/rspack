import fs from "node:fs";
import path from "node:path";

import type {
	ECompilerType,
	ITestContext,
	TCompiler,
	TCompilerOptions
} from "../type";
import { BasicProcessor, type IBasicProcessorOptions } from "./basic";

export interface INormalProcessorOptions<T extends ECompilerType>
	extends IBasicProcessorOptions<T> {
	compilerOptions?: TCompilerOptions<T>;
	root: string;
}

export class NormalProcessor<
	T extends ECompilerType
> extends BasicProcessor<T> {
	constructor(protected _normalOptions: INormalProcessorOptions<T>) {
		super({
			findBundle: (context, options) => {
				const filename = options.output?.filename;
				return typeof filename === "string" ? filename : undefined;
			},
			defaultOptions: NormalProcessor.defaultOptions,
			..._normalOptions
		});
	}

	static defaultOptions<T extends ECompilerType>(
		this: NormalProcessor<T>,
		context: ITestContext
	): TCompilerOptions<T> {
		let testConfig: TCompilerOptions<T> = {};
		const testConfigPath = path.join(context.getSource(), "test.config.js");
		if (fs.existsSync(testConfigPath)) {
			testConfig = require(testConfigPath);
		}
		const TerserPlugin = require("terser-webpack-plugin");
		const terserForTesting = new TerserPlugin({
			parallel: false
		});
		const { root, compilerOptions } = this._normalOptions;
		return {
			context: root,
			entry: `./${path.relative(root, context.getSource())}/`,
			target: compilerOptions?.target || "async-node",
			devtool: compilerOptions?.devtool,
			mode: compilerOptions?.mode || "none",
			optimization: compilerOptions?.mode
				? {
						// emitOnErrors: true,
						minimizer: [terserForTesting],
						...testConfig.optimization
					}
				: {
						removeAvailableModules: true,
						removeEmptyChunks: true,
						mergeDuplicateChunks: true,
						// CHANGE: rspack does not support `flagIncludedChunks` yet.
						// flagIncludedChunks: true,
						sideEffects: true,
						providedExports: true,
						usedExports: true,
						mangleExports: true,
						// CHANGE: rspack does not support `emitOnErrors` yet.
						// emitOnErrors: true,
						concatenateModules: !!testConfig?.optimization?.concatenateModules,
						innerGraph: true,
						// CHANGE: size is not supported yet
						// moduleIds: "size",
						// chunkIds: "size",
						moduleIds: "named",
						chunkIds: "named",
						minimizer: [terserForTesting],
						...compilerOptions?.optimization
					},
			// CHANGE: rspack does not support `performance` yet.
			// performance: {
			// 	hints: false
			// },
			node: {
				__dirname: "mock",
				__filename: "mock"
			},
			cache: compilerOptions?.cache && {
				// cacheDirectory,
				...(compilerOptions.cache as any)
			},
			output: {
				pathinfo: "verbose",
				path: context.getDist(),
				filename: compilerOptions?.module ? "bundle.mjs" : "bundle.js"
			},
			resolve: {
				modules: ["web_modules", "node_modules"],
				mainFields: ["webpack", "browser", "web", "browserify", "main"],
				aliasFields: ["browser"],
				extensions: [".webpack.js", ".web.js", ".js", ".json"]
			},
			resolveLoader: {
				modules: ["web_loaders", "web_modules", "node_loaders", "node_modules"],
				mainFields: ["webpackLoader", "webLoader", "loader", "main"],
				extensions: [
					".webpack-loader.js",
					".web-loader.js",
					".loader.js",
					".js"
				]
			},
			module: {
				rules: []
			},
			plugins: (compilerOptions?.plugins || [])
				// @ts-ignore
				.concat(testConfig.plugins || [])
				.concat(function (this: TCompiler<T>) {
					this.hooks.compilation.tap("TestCasesTest", compilation => {
						const hooks: never[] = [
							// CHANGE: the following hooks are not supported yet, so comment it out
							// "optimize",
							// "optimizeModules",
							// "optimizeChunks",
							// "afterOptimizeTree",
							// "afterOptimizeAssets"
						];

						for (const hook of hooks) {
							(compilation.hooks[hook] as any).tap("TestCasesTest", () =>
								(compilation as any).checkConstraints()
							);
						}
					});
				}),
			experiments: {
				css: true,
				rspackFuture: {
					bundlerInfo: {
						force: false
					}
				},
				asyncWebAssembly: true,
				topLevelAwait: true,
				// CHANGE: rspack does not support `backCompat` yet.
				// backCompat: false,
				// CHANGE: Rspack enables `css` by default.
				// Turning off here to fallback to webpack's default css processing logic.
				...(compilerOptions?.module ? { outputModule: true } : {})
			}
			// infrastructureLogging: compilerOptions?.cache && {
			//   debug: true,
			//   console: createLogger(infraStructureLog)
			// }
		} as TCompilerOptions<T>;
	}

	static overrideOptions<T extends ECompilerType>(
		options: TCompilerOptions<T>
	) {
		if (!global.printLogger) {
			options.infrastructureLogging = {
				level: "error"
			};
		}
	}
}
