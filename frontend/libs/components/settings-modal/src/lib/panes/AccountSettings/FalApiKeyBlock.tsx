import { Button } from "@storyteller/ui-button";
import { Input } from "@storyteller/ui-input";
import { useEffect, useState } from "react";
import { LoaderCircleIcon } from "lucide-react";
import { ProviderList, ProviderSetApiKey } from "@storyteller/tauri-api";

export const FalApiKeyBlock = () => {
  const [keyStart, setKeyStart] = useState<string | undefined>(undefined);
  const [draftKey, setDraftKey] = useState("");
  const [isSaving, setIsSaving] = useState(false);

  const refresh = async () => {
    try {
      const result = await ProviderList();
      const fal = result.payload?.providers?.find(
        (p) => p.provider_credential === "fal_api_key"
      );
      setKeyStart(
        fal?.has_credentials ? fal.maybe_details?.maybe_key_start : undefined
      );
    } catch (e) {
      console.error("Error fetching fal API key status", e);
    }
  };

  useEffect(() => {
    refresh();
  }, []);

  const saveKey = async () => {
    const key = draftKey.trim();
    if (!key) return;
    setIsSaving(true);
    try {
      await ProviderSetApiKey("fal_api_key", key);
      setDraftKey("");
      await refresh();
    } catch (e) {
      console.error("Error saving fal API key", e);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="flex flex-col gap-2">
      <div className="flex min-w-0 flex-col gap-0.5">
        <p className="font-mono text-[11px] font-semibold uppercase tracking-[0.12em]">
          fal API key
        </p>
        <p className="truncate text-sm font-medium text-base-fg/80">
          {keyStart ? `Saved (${keyStart})` : "Not set"}
        </p>
      </div>
      <div className="flex items-center gap-3">
        <Input
          type="password"
          value={draftKey}
          onChange={(e) => setDraftKey(e.target.value)}
          placeholder={keyStart ? "Paste a new key to replace it" : "Paste your fal API key"}
        />
        <Button
          variant="primary"
          className="h-9 shrink-0 px-3"
          onClick={saveKey}
          disabled={isSaving || !draftKey.trim()}
        >
          {isSaving ? <LoaderCircleIcon className="animate-spin text-sm" /> : "Save"}
        </Button>
      </div>
    </div>
  );
};
