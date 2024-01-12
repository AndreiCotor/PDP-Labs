package message;

public class PrepMessage extends BaseMessage {
    private String variable;
    private int timestamp;

    public PrepMessage(String variable, int timestamp) {
        this.timestamp = timestamp;
        this.variable = variable;
    }

    public void setTimestamp(int timestamp) {
        this.timestamp = timestamp;
    }

    public int getTimestamp() {
        return timestamp;
    }

    public String getVariable() {
        return variable;
    }

    public void setVariable(String variable) {
        this.variable = variable;
    }
}
