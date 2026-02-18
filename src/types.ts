export interface TreeNode {
  name: string;
  path: string;
  is_dir: boolean;
  children: TreeNode[];
}

export interface DependencyStatus {
  name: string;
  installed: boolean;
  version: string | null;
}

export interface SetupStatus {
  dependencies_ok: boolean;
  store_exists: boolean;
  needs_setup: boolean;
  first_run: boolean;
  dependencies: DependencyStatus[];
}

export interface StoreInfo {
  store_path: string;
  gpg_id: string | null;
  has_git: boolean;
  git_remote: string | null;
}

export interface GpgKey {
  key_id: string;
  uid: string;
  fingerprint: string;
}
