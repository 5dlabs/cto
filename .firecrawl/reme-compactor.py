"""Compactor module for memory compaction operations."""

from agentscope.agent import ReActAgent
from agentscope.message import Msg

from ..utils import AsMsgHandler
from ....core.op import BaseOp
from ....core.utils import get\_logger

logger = get\_logger()

class Compactor(BaseOp):
 """Compactor class for compacting memory messages."""

 def \_\_init\_\_(
 self,
 memory\_compact\_threshold: int,
 console\_enabled: bool = False,
 \*\*kwargs,
 ):
 super().\_\_init\_\_(\*\*kwargs)
 self.memory\_compact\_threshold: int = memory\_compact\_threshold
 self.console\_enabled: bool = console\_enabled

 async def execute(self):
 messages: list\[Msg\] = self.context.get("messages", \[\])
 previous\_summary: str = self.context.get("previous\_summary", "")

 if not messages:
 return ""

 msg\_handler = AsMsgHandler(self.as\_token\_counter)
 before\_token\_count = await msg\_handler.count\_msgs\_token(messages)
 history\_formatted\_str: str = await msg\_handler.format\_msgs\_to\_str(
 messages=messages,
 memory\_compact\_threshold=self.memory\_compact\_threshold,
 )
 after\_token\_count = await msg\_handler.count\_str\_token(history\_formatted\_str)
 logger.info(f"Compactor before\_token\_count={before\_token\_count} after\_token\_count={after\_token\_count}")

 if not history\_formatted\_str:
 logger.warning(f"No history to compact. messages={messages}")
 return ""

 agent = ReActAgent(
 name="reme\_compactor",
 model=self.as\_llm,
 sys\_prompt=self.get\_prompt("system\_prompt"),
 formatter=self.as\_llm\_formatter,
 )
 agent.set\_console\_output\_enabled(self.console\_enabled)

 if previous\_summary:
 prefix: str = self.get\_prompt("update\_user\_message\_prefix")
 suffix: str = self.get\_prompt("update\_user\_message\_suffix")
 user\_message: str = (
 f"\\n{history\_formatted\_str}\\n\\n\\n"
 f"{prefix}\\n\\n"
 f"\\n{previous\_summary}\\n\\n\\n"
 f"{suffix}"
 )
 else:
 user\_message: str = f"\\n{history\_formatted\_str}\\n\\n\\n" + self.get\_prompt(
 "initial\_user\_message",
 )
 logger.info(f"Compactor sys\_prompt={agent.sys\_prompt} user\_message={user\_message}")

 compact\_msg: Msg = await agent.reply(
 Msg(
 name="reme",
 role="user",
 content=user\_message,
 ),
 )

 history\_compact: str = compact\_msg.get\_text\_content()
 logger.info(f"Compactor Result:\\n{history\_compact}")
 return history\_compact