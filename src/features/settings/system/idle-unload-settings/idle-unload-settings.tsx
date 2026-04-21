import { SettingsUI } from '@/components/settings-ui';
import { Typography } from '@/components/typography';
import { MemoryStick } from 'lucide-react';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/select';
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
    const { minutes, setMinutes } = useIdleUnloadState();

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
            </SettingsUI.Description>
            <Select value={String(minutes)} onValueChange={(v) => setMinutes(Number(v))}>
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
