import { useEffect, useState } from 'react';
import { AsBind } from 'as-bind';

export const useWasm = (fileName, imports) => {
    const [state, setState] = useState({
        loaded: false,
        instance: null,
        error: null,
    });
    useEffect(() => {
        const abortController = new AbortController();
        const fetchWasm = async () => {
            try {
                const wasm = await fetch(
                    fileName,
                    { signal: abortController.signal }
                );
                if (!wasm.ok) {
                    throw new Error(`Failed to fetch resource ${fileName}.`);
                }
                const instance = await AsBind.instantiate(wasm, imports);
                if (!abortController.signal.aborted) {
                    setState({ instance, loaded: true, error: null });
                }
            } catch (e) {
                if (!abortController.signal.aborted) {
                    setState({ ...state, error: e });
                }
            }
        };
        fetchWasm();
        return function cleanup() {
            abortController.abort();
        };
    }, [fileName, imports]);
    return state;
}