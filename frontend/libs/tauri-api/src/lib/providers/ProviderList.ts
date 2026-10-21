import { invoke } from "@tauri-apps/api/core";
import { CommandResult } from "../common/CommandStatus";

export interface ProviderListSuccess extends CommandResult {
  payload: ProviderListPayload;
}

export interface ProviderListPayload {
  providers: ProviderListEntry[];
}

export interface ProviderListEntry {
  provider_credential: string;
  credential_type: string;
  has_credentials: boolean;
  maybe_details?: {
    maybe_key_start?: string;
  };
}

// Throws on Network/Tauri errors.
export const ProviderList = async () : Promise<ProviderListSuccess> => {
  return await invoke("provider_list_command") as ProviderListSuccess;
}
