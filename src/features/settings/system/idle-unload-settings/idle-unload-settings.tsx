import { SettingsUI } from '@/components/settings-ui';
import { Typography } from '@/components/typography';
import { MemoryStick } from 'lucide-react';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/select';
import { Checkbox } from '@/components/checkbox';
import { useTranslation } from '@/i18n';
import { useIdleUnloadState } from './hooks/use-idle-unload-state';

const PRESETS = [
    { value: 0, label: 'Never' },
    { value: 5, label: '5 min' },
    { value: 15, label: '15 min' },
    { value: 30, label: '30 min' },
    { value: 60, label: '1 h' },
];

export const IdleUnloadSettings = () => {
    const { t } = useTranslation();
    const { minutes, setMinutes, followOllama, setFollowOllama, ollama, ollamaOptInAvailable } =
        useIdleUnloadState();

    const ollamaLabel = (() => {
        if (!ollama || ollama.raw === null) return '';
        if (ollama.never) return t('never');
        if (ollama.minutes !== null) return `${ollama.minutes} min`;
        return `"${ollama.raw}" ${t('(unparseable)')}`;
    })();

    return (
        <SettingsUI.Item>
            <SettingsUI.Description>
                <Typography.Title className="flex items-center gap-2">
                    <MemoryStick className="w-4 h-4 text-muted-foreground" />
                    {t('Unload model after idle')}
                </Typography.Title>
                <Typography.Paragraph>
                    {t('Free the model from memory when Murmure has been idle. Ignored while Voice Mode is active.')}
                </Typography.Paragraph>
                {ollamaOptInAvailable && (
                    <div className="flex flex-col gap-1 mt-2">
                        <div className="flex items-center gap-2">
                            <Checkbox
                                id="idle-unload-follow-ollama"
                                checked={followOllama}
                                onCheckedChange={(checked) => setFollowOllama(checked === true)}
                                disabled={ollama?.minutes === null && !ollama?.never}
                            />
                            <label
                                htmlFor="idle-unload-follow-ollama"
                                className="text-sm text-muted-foreground cursor-pointer"
                            >
                                {t('Follow OLLAMA_KEEP_ALIVE')} ({ollamaLabel})
                            </label>
                        </div>
                        <p className="text-xs text-muted-foreground/70 pl-6">
                            {t(
                                "Only detected when OLLAMA_KEEP_ALIVE is set at the session level (launchctl on macOS, system env on Windows, user session on Linux). A systemd-service-scoped or shell-rc-only value won't be visible here."
                            )}
                        </p>
                    </div>
                )}
            </SettingsUI.Description>
            <Select
                value={String(minutes)}
                onValueChange={(v) => setMinutes(Number(v))}
                disabled={followOllama && ollamaOptInAvailable}
            >
                <SelectTrigger className="w-[180px]" data-testid="idle-unload-select">
                    <SelectValue />
                </SelectTrigger>
                <SelectContent>
                    {PRESETS.map((preset) => (
                        <SelectItem key={preset.value} value={String(preset.value)}>
                            {t(preset.label)}
                        </SelectItem>
                    ))}
                </SelectContent>
            </Select>
        </SettingsUI.Item>
    );
};
