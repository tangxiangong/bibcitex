if (args is ["--export-fixture", var exportDirectory]) { UpdateFeedTests.ExportFixture(exportDirectory); return; }
if (args is ["--verify-fixture", var verifyDirectory]) { UpdateFeedTests.VerifyFixture(verifyDirectory); return; }
UpdatePolicyTests.Run();
UpdateFeedTests.Run();
await UpdateSourceTests.Run();
var completed = new TaskCompletionSource();
var operation = BibCiTeX.UpdateActivity.Run(() => { completed.Task.GetAwaiter().GetResult(); });
if (!BibCiTeX.UpdateActivity.IsBusy) throw new InvalidOperationException("Queued operations must block update relaunch");
completed.SetResult();
await operation;
if (BibCiTeX.UpdateActivity.IsBusy) throw new InvalidOperationException("Completed operations must release update relaunch");
try { await BibCiTeX.UpdateActivity.Run(() => { throw new IOException("expected"); }); }
catch (IOException) { }
if (BibCiTeX.UpdateActivity.IsBusy) throw new InvalidOperationException("Failed operations must release update relaunch");
Console.WriteLine("Update scheduling, metadata authentication, mirror transport and operation tracking passed.");
