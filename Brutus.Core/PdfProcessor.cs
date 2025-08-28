using PdfSharp.Pdf;
using PdfSharp.Pdf.IO;
using PdfSharp.Pdf.Security;
using System.Reflection;

namespace Brutus.Core;

/// <summary>
/// Handles low-level PDF operations using the PDFsharp library.
/// </summary>
public class PdfProcessor(Logger logger)
{
    /// <summary>
    /// Checks if a PDF file is protected by a user password.
    /// This method relies on PDFsharp throwing a specific exception when trying to open a password-protected file without a password.
    /// </summary>
    /// <param name="filePath">The path to the PDF file.</param>
    /// <returns>True if the file is password protected, otherwise false.</returns>
    public bool IsPasswordProtected(string filePath)
    {
        try
        {
            // The current implementation of PDFsharp for .NET Core doesn't have a direct
            // 'IsPasswordProtected' property. The reliable way to check is to try opening it
            // and catch the specific exception it throws for password-protected files.
            using PdfDocument? document = PdfReader.Open(filePath, PdfDocumentOpenMode.Import);

            // If the above line does not throw, the file is not password protected.
            // The extensive logging from the initial debugging has been left here intentionally
            // in case of future issues with different PDF versions.
            logger.LogInfo("PDF opened without a password. Inspecting SecuritySettings.");
            PdfSecuritySettings? settings = document.SecuritySettings;
            logger.LogInfo($"SecuritySettings object: {settings}");

            foreach (PropertyInfo prop in settings.GetType().GetProperties())
            {
                if (prop.CanRead)
                {
                    logger.LogInfo($"Property: {prop.Name}, Value: {prop.GetValue(settings, null)}");
                }
            }

            return false;
        }
        catch (PdfReaderException ex)
        {
            // This is the expected exception for a password-protected file.
            logger.LogInfo($"Caught PdfReaderException: {ex.Message}. Assuming password protected.");
            return true;
        }
        catch (Exception ex)
        {
            logger.LogError($"An unexpected exception occurred in IsPasswordProtected: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Attempts to open a PDF file using a specific password.
    /// </summary>
    /// <param name="filePath">The path to the PDF file.</param>
    /// <param name="password">The password to try.</param>
    /// <returns>True if the password was correct, otherwise false.</returns>
    public bool TryOpenWithPassword(string filePath, string password)
    {
        try
        {
            // We open in Modify mode as it's a reliable way to force the password check.
            using PdfDocument? document = PdfReader.Open(filePath, password, PdfDocumentOpenMode.Modify);
            // If no exception is thrown, the password is correct.
            return true;
        }
        catch (PdfReaderException)
        {
            // This exception is expected for an incorrect password.
            return false;
        }
        catch (Exception ex)
        {
            logger.LogError($"An unexpected exception occurred in TryOpenWithPassword: {ex.Message}");
            return false;
        }
    }
}