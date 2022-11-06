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

export function usedFn() {}
export function unUsedFn() {}
export const unUsedFn = () => username;

const exporter = {
  usedArrowFn,
  usedBool,
  usedString,
};

export default exporter;
