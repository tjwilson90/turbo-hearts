import nodeResolve from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";
import replace from "@rollup/plugin-replace";

module.exports = {
    input: 'dist/js/index.js',
    output: {
        file: '../../assets/lobby/index.js',
        format: 'iife'
    },
    plugins: [
        replace({
            'process.env.NODE_ENV': JSON.stringify(process.env.NODE_ENV || "development")
        }),
        commonjs({
            namedExports: {
                'node_modules/react/index.js': ['Component', 'PureComponent', 'Fragment', 'Children', 'createElement', 'useMemo', 'useEffect', 'useLayoutEffect', "useContext", "useRef", ,"useReducer"],
                'node_modules/react-is/index.js': ['isValidElementType', 'isContextConsumer'],
                'node_modules/react-dom/index.js': ['render', 'unstable_batchedUpdates']
            },
        }),
        nodeResolve()
    ],
};
