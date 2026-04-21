import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { toast } from 'react-toastify';
import { useTranslation } from '@/i18n';

export const ModelLoadErrorListener = () => {
    const { t } = useTranslation();

    useEffect(() => {
        const unlisten = listen<string>('model-load-error', () => {
            toast.error(t('Failed to reload the transcription model.'), { autoClose: 5000 });
        });

        return () => {
            unlisten.then((fn) => fn());
        };
    }, [t]);

    return null;
};
