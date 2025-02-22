export function calculateAspectRatio(width: number, height: number): { aspectWidth: number; aspectHeight: number } {
  const gcd = (a: number, b: number): number => {
    return b === 0 ? a : gcd(b, a % b);
  };

  const divisor = gcd(width, height);
  const aspectWidth = width / divisor;
  const aspectHeight = height / divisor;

  return { aspectWidth, aspectHeight };
}
