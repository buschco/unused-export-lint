const usedString = "John";
const unUsedString = "John";

const usedArrowFn = () => username;
const unUsedArrowFn = () => username;

const usedBool = true;
const unUsedBool = true;

const usedNamedString = "John";
const unUsedNamedString = "John";

export { usedNamedString, unUsedNamedString };
export const usedDirectExportString = "John";
export const unUsedDirectExportString = "John";
export type usedTypeDirect = { foo: string };
export type unUsedTypeDirect = { foo: string };
type usedType = { foo: string };
type unUsedType = { foo: string };
export type { usedType, unUsedType };
export function usedFnDirect() {}
export function unUsedFnDirect() {}
export const unUsedArrowFnDirect = () => username;

const exporter = {
  usedArrowFn,
  usedBool,
  usedString,
};

export default exporter;
