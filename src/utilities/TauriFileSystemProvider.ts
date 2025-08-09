import {
  Disposable,
  IDisposable,
} from "@codingame/monaco-vscode-api/vscode/vs/base/common/lifecycle";
import { URI } from "@codingame/monaco-vscode-api/vscode/vs/base/common/uri";
import {
  FileSystemProviderCapabilities,
  FileType,
  IFileChange,
  IFileDeleteOptions,
  IFileOverwriteOptions,
  IFileSystemProviderWithFileReadWriteCapability,
  IFileWriteOptions,
  IStat,
  IWatchOptions,
} from "@codingame/monaco-vscode-files-service-override";
import {
  Emitter,
  Event,
} from "@codingame/monaco-vscode-api/vscode/vs/base/common/event";
import * as fs from "@tauri-apps/plugin-fs";

export default class TauriFileSystemProvider
  extends Disposable
  implements IFileSystemProviderWithFileReadWriteCapability
{
  private _onDidChangeFile: Emitter<readonly IFileChange[]>;

  capabilities: FileSystemProviderCapabilities;
  onDidChangeCapabilities: Event<void>;
  onDidChangeFile: Event<readonly IFileChange[]>;
  onDidWatchError?: Event<string> | undefined;

  constructor(readonly: boolean) {
    super();
    this.onDidChangeCapabilities = Event.None;
    this._onDidChangeFile = new Emitter();
    this.onDidChangeFile = this._onDidChangeFile.event;
    this.capabilities =
      FileSystemProviderCapabilities.FileReadWrite |
      FileSystemProviderCapabilities.PathCaseSensitive;
    if (readonly) {
      this.capabilities |= FileSystemProviderCapabilities.Readonly;
    }
  }
  async readFile(resource: URI): Promise<Uint8Array> {
    return await fs.readFile(resource.fsPath);
  }
  async writeFile(resource: URI, content: Uint8Array, opts: IFileWriteOptions) {
    await fs.writeFile(resource.fsPath, content, {
      create: opts.create,
      createNew: !opts.overwrite,
    });
  }

  stat(resource: URI): Promise<IStat> {
    return new Promise(async (resolve, reject) => {
      try {
        let stat = await fs.stat(resource.path);
        let type = stat.isFile ? FileType.File : FileType.Directory;
        if (stat.isSymlink) {
          type = FileType.SymbolicLink;
        }
        let ctime = stat.birthtime?.getMilliseconds() || 0;
        let mtime = stat.mtime?.getMilliseconds() || 0;
        resolve({
          type,
          ctime,
          mtime,
          size: stat.size,
        });
      } catch (error) {
        reject(error);
      }
    });
  }

  // TODO: Implement remaining methods
  // @ts-ignore
  watch(resource: URI, opts: IWatchOptions): IDisposable {
    return Disposable.None;
  }

  // @ts-ignore
  mkdir(resource: URI): Promise<void> {
    throw new Error("Mkdir not implemented.");
  }

  // @ts-ignore
  readdir(resource: URI): Promise<[string, FileType][]> {
    throw new Error("Readdir not implemented.");
  }

  // @ts-ignore
  delete(resource: URI, opts: IFileDeleteOptions): Promise<void> {
    throw new Error("Delete not implemented.");
  }

  // @ts-ignore
  rename(from: URI, to: URI, opts: IFileOverwriteOptions): Promise<void> {
    throw new Error("Rename not implemented.");
  }
}
