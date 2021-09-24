use vino_interface_keyvalue::generated::exists::*;

pub(crate) async fn job(input: Inputs, output: Outputs, context: crate::Context) -> JobResult {
  let mut cmd = redis::Cmd::exists(&input.key);
  let value: u32 = context.run_cmd(&mut cmd).await?;
  if value == 0 {
    output.exists.done(Payload::success(&false))?;
  } else {
    output.exists.done(Payload::success(&true))?;
  };
  Ok(())
}
