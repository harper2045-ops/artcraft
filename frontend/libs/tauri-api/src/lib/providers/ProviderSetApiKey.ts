import { invoke } from "@tauri-apps/api/core";
import { CommandResult } from "../common/CommandStatus";

export type ProviderApiKeyCredential = "fal_api_key" | "replicate_api_key";

export const ProviderSetApiKey = async (
  providerCredential: ProviderApiKeyCredential,
  apiKey: string,
) : Promise<CommandResult> => {
  const result = await invoke("provider_set_api_key_command", {
    request: {
      provider_credential: providerCredential,
      api_key: apiKey,
    }
  });
  return result as CommandResult;
}
