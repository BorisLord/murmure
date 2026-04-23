import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useEffect, useState } from 'react';
import { toast } from 'react-toastify';
import { useTranslation } from '@/i18n';

type OllamaKeepAliveInfo = {
    raw: string | null;
    minutes: number | null;
    never: boolean;
    llm_local_active: boolean;
};

export const useIdleUnloadState = () => {
    const [minutes, setMinutes] = useState<number>(0);
    const [followOllama, setFollowOllama] = useState<boolean>(false);
    const [ollama, setOllama] = useState<OllamaKeepAliveInfo | null>(null);
    const { t } = useTranslation();

    useEffect(() => {
        const loadOllama = async () => {
            try {
                const o = await invoke<OllamaKeepAliveInfo>('detect_ollama_keep_alive');
                setOllama(o);
            } catch (error) {
                console.error('Failed to detect OLLAMA_KEEP_ALIVE:', error);
            }
        };

        const load = async () => {
            try {
                const [m, f] = await Promise.all([
                    invoke<number>('get_idle_unload_minutes'),
                    invoke<boolean>('get_idle_unload_follow_ollama'),
                ]);
                setMinutes(m);
                setFollowOllama(f);
            } catch (error) {
                console.error('Failed to load idle unload setting:', error);
            }
        };
        load();
        loadOllama();

        // LLM Connect changes flip `llm_local_active`; re-detect so the
        // checkbox appears/disappears without reopening the page.
        const unlistenPromise = listen('llm-settings-updated', () => {
            loadOllama();
        });
        return () => {
            unlistenPromise.then((fn) => fn());
        };
    }, []);

    const save = async (value: number) => {
        try {
            await invoke('set_idle_unload_minutes', { minutes: value });
            setMinutes(value);
            toast.success(t('Idle unload updated'));
        } catch (error) {
            console.error('Failed to save idle unload setting:', error);
            toast.error(t('Failed to save idle unload setting'));
        }
    };

    const saveFollowOllama = async (value: boolean) => {
        try {
            await invoke('set_idle_unload_follow_ollama', { follow: value });
            setFollowOllama(value);
            toast.success(t('Idle unload updated'));
        } catch (error) {
            console.error('Failed to save idle unload follow-ollama flag:', error);
            toast.error(t('Failed to save idle unload setting'));
        }
    };

    // The opt-in checkbox is only meaningful when OLLAMA_KEEP_ALIVE is set
    // AND LLM Connect actually runs a local Ollama mode — otherwise the
    // env var can't drive anything.
    const ollamaOptInAvailable =
        ollama !== null && ollama.raw !== null && ollama.llm_local_active;

    return {
        minutes,
        setMinutes: save,
        followOllama,
        setFollowOllama: saveFollowOllama,
        ollama,
        ollamaOptInAvailable,
    };
};
