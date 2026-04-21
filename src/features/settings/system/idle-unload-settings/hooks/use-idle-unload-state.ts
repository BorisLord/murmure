import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';
import { toast } from 'react-toastify';
import { useTranslation } from '@/i18n';

export const useIdleUnloadState = () => {
    const [minutes, setMinutes] = useState<number>(0);
    const { t } = useTranslation();

    useEffect(() => {
        const load = async () => {
            try {
                const value = await invoke<number>('get_idle_unload_minutes');
                setMinutes(value);
            } catch (error) {
                console.error('Failed to load idle unload setting:', error);
            }
        };
        load();
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

    return { minutes, setMinutes: save };
};
