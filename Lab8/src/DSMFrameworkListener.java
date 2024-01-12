import message.*;
import mpi.MPI;

public class DSMFrameworkListener implements Runnable {

    private final DSMFrameworkNotifyBuffer notifyBuffer;

    public DSMFrameworkListener(DSMFrameworkNotifyBuffer notifyBuffer) {
        this.notifyBuffer = notifyBuffer;
    }

    @Override
    public void run() {
        while (true) {
            Object[] messagesObject = new Object[1];

            var status = MPI.COMM_WORLD.Recv(messagesObject, 0, 1, MPI.OBJECT, MPI.ANY_SOURCE, 0);
            BaseMessage message = (BaseMessage) messagesObject[0];
            if (message instanceof CloseMessage) {
                notifyBuffer.flushAll();
                break;
            }  else if (message instanceof PrepMessage prepMessage) {
                notifyBuffer.updateTimestamp(prepMessage.getTimestamp());

                MPI.COMM_WORLD.Send(new Object[]{new TSMessage(notifyBuffer.getTimestamp())}, 0, 1, MPI.OBJECT, status.source, 1);
            }
            else if (message instanceof ActionMessage actionMessage) {
                notifyBuffer.add(actionMessage);
            }
        }
    }
}