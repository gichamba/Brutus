namespace Brutus.Core.Models;

/// <summary>
/// Represents the configuration options for a brute force attack.
/// </summary>
/// <param name="StartPassword">The starting password of the range.</param>
/// <param name="EndPassword">The ending password of the range.</param>
/// <param name="TimeoutPerFile">The timeout in seconds for processing a single file.</param>
public record BruteForceOptions(int StartPassword = 0, int EndPassword = 999999, int TimeoutPerFile = 300);