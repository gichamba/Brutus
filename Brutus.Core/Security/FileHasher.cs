using System.Security.Cryptography;

namespace Brutus.Core.Security
{
    public class FileHasher
    {
        public static string ComputeSha256(string filePath)
        {
            using SHA256? sha256 = SHA256.Create();
            using FileStream? stream = File.OpenRead(filePath);
            byte[]? hash = sha256.ComputeHash(stream);
            return BitConverter.ToString(hash).Replace("-", "").ToLowerInvariant();
        }
    }
}
